<script setup>
import { ref, computed, onUnmounted, watch, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { download } from "@tauri-apps/plugin-upload";
import { save } from "@tauri-apps/plugin-dialog";
import { downloadDir, join } from "@tauri-apps/api/path";
import { ElMessage } from 'element-plus'
import { appCacheDir } from '@tauri-apps/api/path';
import { debounce,throttle  } from 'lodash-es'
// import { set, get } from 'tauri-plugin-cache-api';

const tags = ref("");
const images = ref([]);
const loading = ref(false);
const error = ref("");
const selectedImageIndex = ref(null);
const currentImageUrl = ref(null);
const loadingImage = ref(false);
const urlCache = ref({}); // URL -> blob URL 的缓存

const currentPage = ref(1);
const pageSize = ref(50);
const total = ref(0);
const downloadFilePath = ref("");

const nsfwModel = ref("");

const isWaterfall = ref(false);

const selectedImage = computed(() => {
  if (
    selectedImageIndex.value === null ||
    !images.value[selectedImageIndex.value]
  ) {
    return null;
  }
  return images.value[selectedImageIndex.value];
});

const showModal = computed(() => selectedImageIndex.value !== null);

onMounted(async () => {
  fetchPosts();
  let defaultDownloadDir = localStorage.getItem("downloadFilePath");
  if (!defaultDownloadDir) {
    defaultDownloadDir = await downloadDir();
    localStorage.setItem("downloadFilePath", defaultDownloadDir);
    downloadFilePath.value = defaultDownloadDir;
  }else {
    downloadFilePath.value = defaultDownloadDir;
  }

  const appCachePath = await appCacheDir();
  console.log("App Cache Directory:", appCachePath);


});
const scrollToSection = (id) => {
  const element = document.getElementById(id)
  if (element) {
    element.scrollIntoView({
      behavior: 'smooth',  // 平滑滚动
      block: 'start',       // 垂直对齐方式：start, center, end, nearest
      inline: 'nearest'     // 水平对齐方式
    })
  }
}

function openImageModal(index) {
  selectedImageIndex.value = index;
}

function closeModal() {
  selectedImageIndex.value = null;
}

function nextImage() {
  if (selectedImageIndex.value === null) return;
  const nextIndex = (selectedImageIndex.value + 1) % images.value.length;
  selectedImageIndex.value = nextIndex;
}

function prevImage() {
  if (selectedImageIndex.value === null) return;
  const prevIndex =
    (selectedImageIndex.value - 1 + images.value.length) % images.value.length;
  selectedImageIndex.value = prevIndex;
}

async function loadImage(url) {
  if (!url) return null;

  // 检查内存中的 blob URL 缓存
  if (urlCache.value[url]) {
    return urlCache.value[url];
  }

  // // 检查插件缓存
  // let cachedBytes = null;
  // try {
  //   cachedBytes = await get(url);
  // } catch (e) {
  //   console.warn('Cache get error:', e);
  // }
  // if (cachedBytes) {
  //   // cachedBytes 是数字数组，转换为 Uint8Array
  //   const uint8Array = new Uint8Array(cachedBytes);
  //   const blob = new Blob([uint8Array]);
  //   const blobUrl = URL.createObjectURL(blob);
  //   urlCache.value[url] = blobUrl;
  //   return blobUrl;
  // }

  // 从网络获取
  loadingImage.value = true;
  try {
    // const bytes = await invoke("fetch_image_as_bytes", { url });  
    // const uint8Array = new Uint8Array(bytes);
    // const blob = new Blob([uint8Array]);
    // const blobUrl = URL.createObjectURL(blob);
    // urlCache.value[url] = blobUrl;
    const base64 = await invoke("fetch_image_as_base64", { url });
    let base64Img = `data:image/jpeg;base64,${base64}`;
    // 存储到插件缓存，设置 TTL 7天（604800秒）
    // await set(url, bytes, { ttl: 604800 });
    // 创建 blob URL
    urlCache.value[url] = base64Img;
    return  base64Img;
  } catch (err) {
    console.error('Failed to load image:', err);
    return null;
  } finally {
    loadingImage.value = false;
  }
}

async function loadCurrentImage() {
  currentImageUrl.value = null;
  const url = selectedImage.value.sample_url;
  if (!url) {
    return;
  }

  const imgUrl = await loadImage(url);
  if (imgUrl) {
    currentImageUrl.value = imgUrl;
  } else {
    currentImageUrl.value = null;
  }
}
async function downloadFile(img) {
  img.loading = true; // 开始下载，设置 loading 状态
  let picture = img.value || img; // 兼容直接传入图片对象或ref对象
  // 关键修复：await join 的返回值
  const savePath = await join(
    downloadFilePath.value,
    picture.md5 + "." + picture.file_ext
  );
  await download(
    picture.largeUrl, // 文件下载 URL
    savePath, // 本地保存路径
    { "User-Agent": "Tauri App" } // 可选的请求头（可选）
  );
  ElMessage.success("下载成功！");
  img.loading = false; // 下载完成，重置 loading 状态
}

// 监听 selectedImageIndex 变化，加载图片
watch(selectedImageIndex, async (newIndex) => {
  if (newIndex === null) {
    currentImageUrl.value = null;
    loadingImage.value = false;
    return;
  }
  await loadCurrentImage();
});

// 键盘事件监听
function handleKeydown(event) {
  if (!showModal.value) return;

  switch (event.key) {
    case "Escape":
      closeModal();
      break;
    case "ArrowRight":
      nextImage();
      break;
    case "ArrowLeft":
      prevImage();
      break;
  }
}

// 添加全局键盘事件监听
document.addEventListener("keydown", handleKeydown);

// 组件卸载时移除事件监听并清理 blob URL
onUnmounted(() => {
  document.removeEventListener("keydown", handleKeydown);
  // 撤销所有 blob URL 防止内存泄漏
  Object.values(urlCache.value).forEach(url => {
    URL.revokeObjectURL(url);
  });
  urlCache.value = {};
});

const handleSearch = throttle(() => {
  currentPage.value = 1
  fetchPosts();
},5000)
async function fetchPosts() {
  loading.value = true;
  error.value = "";
  // images.value = [];
  let tagsStr = tags.value;
  if (nsfwModel.value) {
    tagsStr = tagsStr ? `${tagsStr}+${nsfwModel.value}` : nsfwModel.value;
  }
  try {
    const res = await invoke("fetch_posts", {
      tags: tagsStr,
      limit: pageSize.value,
      page: currentPage.value,
    });
    // console.log(res);
    total.value = res.count || 0; // 如果后端返回了总数，更新 total
    // 将posts转换为images格式
    let arr = res.post.map((post) => {
        // 选择最佳图片URL：优先使用sample_url，然后file_url，最后preview_url
        let imageUrl = post.preview_url;

        // 如果URL以https开头且遇到SSL问题，可以尝试转换为http
        // 注意：这可能会降低安全性，但可以解决某些证书问题
        // const fallbackUrl = imageUrl.replace('https://', 'http://');

        return{
          ...post,
          src: imageUrl,
          alt: post.tags,
          created_at: formatTimestamp(post.created_at, "datetime"),
          dimensions: `${post.width}x${post.height}`,
          // 大图URL用于模态框显示
          largeUrl: post.file_url,
          loading: false
        }
      })
    if(isWaterfall.value) {
      images.value = images.value.concat(arr)

    }else {
      images.value = arr
      scrollToSection("imagesDev");
    }
    ElMessage.success("查询成功！");
  } catch (err) {
    error.value = err.message || String(err);
  } finally {
    loading.value = false;
  }
}
function formatTimestamp(timestamp, format = "time") {
  const date = new Date(timestamp * 1000);

  const pad = (num) => num.toString().padStart(2, "0");

  const hours = pad(date.getHours());
  const minutes = pad(date.getMinutes());
  const seconds = pad(date.getSeconds());
  const year = date.getFullYear();
  const month = pad(date.getMonth() + 1);
  const day = pad(date.getDate());

  switch (format) {
    case "time":
      return `${hours}:${minutes}:${seconds}`;
    case "date":
      return `${year}-${month}-${day}`;
    case "datetime":
      return `${year}-${month}-${day} ${hours}:${minutes}:${seconds}`;
    case "compact":
      return `${year}${month}${day}${hours}${minutes}${seconds}`;
    default:
      return `${hours}:${minutes}:${seconds}`;
  }
}

function handleImageError(event, index) {
  console.warn(`Failed to load image at index ${index}:`, event.target.src);

  const img = images.value[index];
  if (!img) return;

  // 尝试备用URL
  if (img.fallbackUrl && img.fallbackUrl !== img.src) {
    event.target.src = img.fallbackUrl;
    return;
  }

  // 如果仍然失败，尝试将HTTPS转换为HTTP
  if (img.src && img.src.startsWith("https://")) {
    const httpUrl = img.src.replace("https://", "http://");
    event.target.src = httpUrl;
    return;
  }

  // 最后使用占位符图片
  event.target.src =
    "https://via.placeholder.com/200x150?text=Image+Failed+to+Load";
}
function handleTags(tag) {
  tags.value = tag;
  fetchPosts();
}


const handleSizeChange = (val) => {
  pageSize.value = val;
  fetchPosts();
};
const handleCurrentChange = (val) => {
  currentPage.value = val;
  fetchPosts();
};

const waterfallLoad = throttle(() =>{
  console.log('waterfallLoad')
    currentPage.value += 1;
    fetchPosts();
},10000)

</script>

<template>
  <main class="container" id="imagesDev" >
    <div class="section">
      <div class="search-container">
        <el-form inline label-width="auto">
          <el-form-item label="Tags">
            <el-input v-model="tags" placeholder="Enter tags (e.g., cute cat)" />
          </el-form-item>
          <el-form-item label="分级">
            <el-select v-model="nsfwModel" placeholder="Select rating" style="width: 120px">
              <el-option value="" label="ALL"></el-option>
              <el-option value="rating:s" label="S"></el-option>
              <el-option value="rating:q" label="Q"></el-option>
              <el-option value="rating:e" label="E"></el-option>
            </el-select>
          </el-form-item>
          <el-form-item>
            <el-button icon="search" type="primary" @click="handleSearch" :loading="loading">
              {{ loading ? "加载中..." : "搜索" }}
            </el-button>
            <el-button @click="isWaterfall = !isWaterfall" type="info">
              {{ isWaterfall ? "网格视图" : "瀑布流" }}
            </el-button>
            <!-- <el-switch v-model="nsfwModel"></el-switch> -->
          </el-form-item>
        </el-form>
      </div>
    

      <div v-if="error" class="error-message">
        {{ error }}
      </div>

      <div  class="images-container" v-loading="loading" v-if="!isWaterfall">
        <div class="images-grid">
          <div v-for="(img, index) in images" :key="index" class="image-item">
            <!-- {{ img.src }} -->
            <img
              :src="img.src"
              :alt="img.alt"
              class="extracted-image"
              @error="handleImageError($event, index)"
              @click="openImageModal(index)"
              style="cursor: pointer"
            />
            <div class="image-meta" >
              <span class="image-rating" :class="'rating-' + img.rating">{{
                img.rating
              }}</span>
              <span class="image-score"><i>尺寸:</i>{{img.dimensions}};<i>时间:</i>{{ img.created_at }}</span>
             
            </div>
            <div class="image-tags" >
              <el-tag @click="handleTags(tag)" type="primary" v-for="tag in img.tags.split(' ')" :key="tag">{{ tag }}</el-tag>
            </div>
            <div class="image-download" >
              <el-button type="primary" round :loading="img.loading" icon="Download" @click="downloadFile(img)"></el-button>
            </div>
          </div>
        </div>
      </div>
       <el-scrollbar 
        v-else
        height="calc(100vh - 85px)"
        :distance="10"
        @end-reached="waterfallLoad"
  >
      <div  class="images-container waterfall" >
        <div class="images-grid " >
          <div v-for="(img, index) in images" :key="index" class="image-item">
            <img
              :src="img.src"
              :alt="img.alt"
              class="extracted-image"
              @error="handleImageError($event, index)"
              @click="openImageModal(index)"
              style="cursor: pointer"
            />
            <div class="image-download" >
              <el-button type="primary" round :loading="img.loading" icon="Download" @click="downloadFile(img)"></el-button>
            </div>
          </div>
        </div>
      </div>
      </el-scrollbar>


      <div v-if="images.length === 0 && !loading" class="no-images-message">
        No images found.
      </div>
      <div class="pagination-container" v-if="!isWaterfall" >
         <el-pagination
          v-model:current-page="currentPage"
          v-model:page-size="pageSize"
          :page-sizes="[20,50, 100, 200]"
          background
          prev-text="上一页"
          next-text="下一页"
          layout="total, sizes, prev, pager, next, jumper"
          :total="total"
          @size-change="handleSizeChange"
          @current-change="handleCurrentChange"
        />
      </div>
     
    </div>

    <!-- 图片模态框 -->
    <div v-if="showModal" class="image-modal" @click.self="closeModal">
      <div class="modal-content">
        <span class="modal-close" @click="closeModal">×</span>
        <div class="modal-image-container">
          <div
            v-if="loadingImage"
            class="modal-loading"
            v-loading="loadingImage"
          >
            <img
              :src="images[selectedImageIndex].src"
              :alt="selectedImage.alt"
              class="modal-image"
            />
          </div>
          <div v-else-if="currentImageUrl" class="modal-image-wrapper">
            <img
              :src="currentImageUrl"
              :alt="selectedImage.alt"
              class="modal-image"
            />
          </div>
          <div v-else class="modal-error">Failed to load image.</div>
        </div>
        <div class="modal-navigation">
          <span class="nav-button prev-button" @click="prevImage">‹</span>
          <span class="nav-button next-button" @click="nextImage">›</span>
        </div>
      </div>
    </div>
  </main>
</template>

<style>
:root {
  font-family: Inter, Avenir, Helvetica, Arial, sans-serif;
  font-size: 16px;
  line-height: 24px;
  font-weight: 400;

  color: #0f0f0f;
  background-color: #f6f6f6;

  font-synthesis: none;
  text-rendering: optimizeLegibility;
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
  -webkit-text-size-adjust: 100%;
}

.container {
  margin: 0;
  display: flex;
  flex-direction: column;
  justify-content: center;
}

.row {
  display: flex;
  justify-content: center;
}

.section {
  border-radius: 8px;
  box-shadow: 0 2px 10px rgba(0, 0, 0, 0.1);
}

.error-message {
  color: #e53935;
  margin-top: 10px;
  padding: 10px;
  background-color: rgba(229, 57, 53, 0.1);
  border-radius: 4px;
}

.html-container {
  margin-top: 20px;
  text-align: left;
}

pre {
  max-height: 400px;
  overflow: auto;
  padding: 15px;
  background-color: #f5f5f5;
  border-radius: 4px;
  font-size: 14px;
  line-height: 1.4;
}

.parsing-message {
  margin-top: 20px;
  padding: 10px;
  background-color: #e3f2fd;
  border-radius: 4px;
  color: #1565c0;
  font-weight: 500;
}

.images-container {
  margin: 10px 0;
  padding: 0 10px;

}

.images-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(300px, 1fr));
  gap: 20px;
  margin-top: 20px;
}

.waterfall .images-grid {
  display: block;
  column-count: 8;
  column-gap: 5px;
}

.waterfall .image-item {
  break-inside: avoid;
  margin-bottom: 5px;
}


.image-item {
  border: 1px solid #e0e0e0;
  border-radius: 8px;
  padding: 5px;
  background-color: white;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.05);
  transition: transform 0.2s, box-shadow 0.2s;
  position: relative;
}

.image-item:hover {
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
}

.extracted-image {
  max-width: 100%;
  height: 200px;
  object-fit: contain;
  border-radius: 4px;
  display: block;
  margin: 0 auto 5px;
}
.waterfall .extracted-image {
  width: 100%;
  height: auto;
}

.image-info {
  font-size: 14px;
  font-weight: 500;
  margin: 10px 0 5px;
  color: #333;
}

.image-meta {
  display: flex;
  justify-content: space-between;
  align-items: center;
  font-size: 12px;
}

.image-rating {
  border-radius: 4px;
  font-weight: bold;
  width: 22px;
  height: 22px;
  line-height: 22px;
  text-align: center;
  text-transform: uppercase;
}

.rating-s {
  background-color: #4caf50;
  color: white;
}

.rating-q {
  background-color: #ff9800;
  color: white;
}

.rating-e {
  background-color: #f44336;
  color: white;
}

.image-score {
  color: #666;
  font-size: 12px;
}
.image-score i {
  font-style: normal;
  font-weight: bold;
  margin-left: 5px;
}

.image-dimensions {
  color: #666;
}

.image-src {
  font-size: 12px;
  color: #666;
  word-break: break-all;
  margin: 0;
  max-height: 60px;
  overflow: auto;
}

.no-images-message {
  margin-top: 20px;
  padding: 15px;
  background-color: #f5f5f5;
  border-radius: 4px;
  color: #666;
  text-align: center;
}

/* 图片模态框样式 */
.image-modal {
  position: fixed;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  background-color: rgba(0, 0, 0, 0.9);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 2000;
  padding: 20px;
  box-sizing: border-box;
}

.modal-content {
  position: relative;
  width: 90%;
  height: 90%;
  background-color: white;
  border-radius: 12px;
  display: flex;
  flex-direction: column;
  box-shadow: 0 10px 40px rgba(0, 0, 0, 0.5);
}

.modal-close {
  position: absolute;
  top: 15px;
  right: 15px;
  width: 40px;
  height: 40px;
  border-radius: 50%;
  background-color: rgba(0, 0, 0, 0.7);
  color: white;
  border: none;
  font-size: 24px;
  font-weight: bold;
  cursor: pointer;
  z-index: 10;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: background-color 0.2s;
}

.modal-close:hover {
  background-color: rgba(0, 0, 0, 0.9);
}

.modal-image-container {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  overflow: hidden;
  padding: 20px;
  height: 80vh;
  background-color: #f5f5f5;
}

.modal-loading,
.modal-error {
  display: flex;
  align-items: center;
  justify-content: center;
  height: 300px;
  color: #666;
  font-size: 18px;
}

.modal-image-wrapper {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 100%;
  height: 80vh;
}

.modal-image {
  width: 1000px;
  height: 100%;
  object-fit: contain;
  border-radius: 8px;
}

.modal-info {
  padding: 20px;
  background-color: white;
  border-top: 1px solid #e0e0e0;
}

.modal-meta {
  display: flex;
  gap: 15px;
  align-items: center;
  margin-bottom: 10px;
  font-size: 14px;
}

.modal-rating {
  padding: 4px 8px;
  border-radius: 4px;
  font-weight: bold;
  text-transform: uppercase;
  font-size: 12px;
}

.modal-score {
  color: #666;
}

.modal-dimensions {
  color: #666;
}

.modal-current-index {
  margin: 0;
  font-size: 14px;
  color: #888;
  text-align: center;
}

.modal-navigation {
  position: absolute;
  top: 50%;
  left: 0;
  right: 0;
  display: flex;
  justify-content: space-between;
  transform: translateY(-50%);
  pointer-events: none;
  padding: 0 20px;
}

.nav-button {
  pointer-events: auto;
  width: 50px;
  height: 50px;
  border-radius: 50%;
  background-color: rgba(0, 0, 0, 0.7);
  color: white;
  border: none;
  font-size: 30px;
  font-weight: bold;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: background-color 0.2s;
}

.nav-button:hover {
  background-color: rgba(0, 0, 0, 0.9);
}

/* 合并所有暗色主题样式 */
@media (prefers-color-scheme: dark) {
  :root {
    color: #f6f6f6;
    background-color: #2f2f2f;
  }

  .section {
    background-color: rgba(30, 30, 30, 0.5);
  }

  .image-item {
    background-color: #2d2d2d;
    border-color: #404040;
  }

  .image-info {
    color: #f0f0f0;
  }

  .image-meta {
    color: #ccc;
  }

  .image-score {
    color: #aaa;
  }

  .image-dimensions {
    color: #aaa;
  }

  .image-src {
    color: #bbb;
  }

  .parsing-message {
    background-color: #1a237e;
    color: #90caf9;
  }

  .no-images-message {
    background-color: #2d2d2d;
    color: #bbb;
  }

  pre {
    background-color: #2d2d2d;
    color: #f8f8f2;
  }

  .error-message {
    background-color: rgba(229, 57, 53, 0.15);
  }

  button:active {
    background-color: #0f0f0f69;
  }

  /* 暗色主题下的模态框样式 */
  .image-modal {
    background-color: rgba(0, 0, 0, 0.95);
  }

  .modal-content {
    background-color: #2d2d2d;
    border-color: #404040;
  }

  .modal-image-container {
    background-color: #1a1a1a;
  }

  .modal-loading,
  .modal-error {
    color: #aaa;
  }

  .modal-info {
    background-color: #2d2d2d;
    border-top-color: #404040;
  }

  .modal-tags {
    color: #f0f0f0;
  }

  .modal-meta {
    color: #ccc;
  }

  .modal-score,
  .modal-dimensions {
    color: #aaa;
  }

  .modal-current-index {
    color: #888;
  }

  .modal-close,
  .nav-button {
    background-color: rgba(255, 255, 255, 0.2);
  }

  .modal-close:hover,
  .nav-button:hover {
    background-color: rgba(255, 255, 255, 0.3);
  }
}
.download, .info {
  color: #2196f3;
  cursor: pointer;
  font-size: 12px;
}
.image-tags {
  display: flex;
  flex-wrap: wrap;
  gap: 5px;
  justify-content: flex-start;
  margin-top: 5px;
}
.image-tags span {
  cursor: pointer;
}
.image-download {
  position: absolute;
  right: 10px;
  top: 10px;
  z-index: 1000;
  opacity: 0;
  transition: all .3s;
}
.image-item:hover .image-download{
  opacity: 1;
}
.pagination-container {
  position: sticky;
  bottom: 0;
  background-color: white;
  padding: 10px;
  box-shadow: 0 -2px 10px rgba(0, 0, 0, 0.1);
  z-index: 1001;
}
.search-container {
  position: sticky;
  top: 0;
  background-color: white;
  padding: 10px;
  z-index: 1001;
  box-shadow: 0 2px 10px rgba(0, 0, 0, 0.1);
}
.search-container .el-form-item--small {
  margin-bottom: 0;
}
</style>
