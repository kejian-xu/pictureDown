<script setup>
import { ref, computed, onUnmounted, watch, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { download } from "@tauri-apps/plugin-upload";
import { save } from "@tauri-apps/plugin-dialog";
import { downloadDir, join } from "@tauri-apps/api/path";
import { ElMessage } from 'element-plus'

const tags = ref("");
const images = ref([]);
const loading = ref(false);
const error = ref("");
const selectedImageIndex = ref(null);
const currentImageBase64 = ref(null);
const loadingImage = ref(false);
const imageCache = ref({}); // URL -> base64 字符串的缓存

const currentPage = ref(1);
const pageSize = ref(20);

const nsfwModel = ref(false);

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

async function loadImageAsBase64(url) {
  if (!url) return null;

  // 检查缓存
  if (imageCache.value[url]) {
    console.log("Using cached image for URL:", url);
    return imageCache.value[url];
  }

  console.log("Fetching image as base64 from URL:", url);
  loadingImage.value = true;
  try {
    const base64String = await invoke("fetch_image_as_base64", { url });
    // 存储到缓存
    imageCache.value[url] = base64String;
    console.log(
      "Image fetched successfully, base64 length:",
      base64String.length
    );
    return base64String;
  } catch (err) {
    console.error("Failed to fetch image as base64:", err);
    return null;
  } finally {
    loadingImage.value = false;
  }
}

async function loadCurrentImage() {
  currentImageBase64.value = null;
  const url = selectedImage.value.largeUrl;
  if (!url) {
    return;
  }

  const base64 = await loadImageAsBase64(url);
  if (base64) {
    currentImageBase64.value = `data:image/jpeg;base64,${base64}`;
  } else {
    currentImageBase64.value = null;
  }
}
async function downloadFile(img) {
  let picture = img.value || img; // 兼容直接传入图片对象或ref对象
  console.log("picture:", picture.largeUrl);
  const defaultDownloadDir = await downloadDir();
  console.log("Default download directory:", defaultDownloadDir);
  // 关键修复：await join 的返回值
  const savePath = await join(
    defaultDownloadDir,
    picture.md5 + "." + picture.file_ext
  );
  try {
    await download(
      picture.largeUrl, // 文件下载 URL
      savePath, // 本地保存路径
      { "User-Agent": "Tauri App" } // 可选的请求头（可选）
    );
  } catch (error) {
    console.error("下载失败:", error);
  }
}

// 监听 selectedImageIndex 变化，加载图片
watch(selectedImageIndex, async (newIndex) => {
  if (newIndex === null) {
    currentImageBase64.value = null;
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

// 组件卸载时移除事件监听
onUnmounted(() => {
  document.removeEventListener("keydown", handleKeydown);
});

async function fetchPosts() {
  loading.value = true;
  error.value = "";
  images.value = [];

  try {
    const res = await invoke("fetch_posts", {
      tags: tags.value,
      limit: pageSize.value,
      page: currentPage.value,
    });
    console.log(res);
    // 将posts转换为images格式
    res.post.forEach((post) => {
      // 选择最佳图片URL：优先使用sample_url，然后file_url，最后preview_url
      let imageUrl = post.preview_url;

      // 如果URL以https开头且遇到SSL问题，可以尝试转换为http
      // 注意：这可能会降低安全性，但可以解决某些证书问题
      // const fallbackUrl = imageUrl.replace('https://', 'http://');

      images.value.push({
        ...post,
        src: imageUrl,
        alt: post.tags,
        created_at: formatTimestamp(post.created_at, "datetime"),
        dimensions: `${post.width}x${post.height}`,
        // 大图URL用于模态框显示
        largeUrl: post.file_url,
      });
    });
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
    console.log(`Trying fallback URL for image ${index}: ${img.fallbackUrl}`);
    event.target.src = img.fallbackUrl;
    return;
  }

  // 如果仍然失败，尝试将HTTPS转换为HTTP
  if (img.src && img.src.startsWith("https://")) {
    const httpUrl = img.src.replace("https://", "http://");
    console.log(`Trying HTTP URL for image ${index}: ${httpUrl}`);
    event.target.src = httpUrl;
    return;
  }

  // 最后使用占位符图片
  event.target.src =
    "https://via.placeholder.com/200x150?text=Image+Failed+to+Load";
}
onMounted(() => {
  fetchPosts();
});

const handleSizeChange = (val) => {
  console.log(`${val} items per page`);
  pageSize.value = val;
  fetchPosts();
};
const handleCurrentChange = (val) => {
  console.log(`current page: ${val}`);
  currentPage.value = val;
  fetchPosts();
};
</script>

<template>
  <main class="container">
    <div class="section">
      <el-form inline label-width="auto">
        <el-form-item label="Tags">
          <el-input v-model="tags" placeholder="Enter tags (e.g., cute cat)" />
        </el-form-item>
        <el-form-item>
          <el-button type="primary" @click="fetchPosts" :loading="loading">
            {{ loading ? "Fetching..." : "Fetch Images" }}
          </el-button>
          <!-- <el-switch v-model="nsfwModel"></el-switch> -->
        </el-form-item>
      </el-form>

      <div v-if="error" class="error-message">
        {{ error }}
      </div>

      <div v-if="images.length > 0" class="images-container">
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
            <div class="image-meta">
              <span class="image-rating" :class="'rating-' + img.rating">{{
                img.rating
              }}</span>
              <span class="image-score">Time: {{ img.created_at }}</span>
              <span class="download" @click="downloadFile(img)">下载原图</span>
            </div>
          </div>
        </div>
      </div>

      <div v-if="images.length === 0 && !loading" class="no-images-message">
        No images found.
      </div>
      <el-pagination
        v-model:current-page="currentPage"
        v-model:page-size="pageSize"
        layout="total, size, prev, pager, next, jumper"
        :total="1000"
        @size-change="handleSizeChange"
        @current-change="handleCurrentChange"
      />
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
          <div v-else-if="currentImageBase64" class="modal-image-wrapper">
            <img
              :src="currentImageBase64"
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
  text-align: center;
}

.row {
  display: flex;
  justify-content: center;
}

.section {
  margin-top: 40px;
  padding: 20px;
  border-radius: 8px;
  background-color: rgba(255, 255, 255, 0.5);
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
  margin: 20px 0;
}

.images-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(300px, 1fr));
  gap: 20px;
  margin-top: 20px;
}

.image-item {
  border: 1px solid #e0e0e0;
  border-radius: 8px;
  padding: 15px;
  background-color: white;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.05);
  transition: transform 0.2s, box-shadow 0.2s;
}

.image-item:hover {
  transform: translateY(-2px);
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
}

.extracted-image {
  max-width: 100%;
  max-height: 200px;
  object-fit: contain;
  border-radius: 4px;
  display: block;
  margin: 0 auto 10px;
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
  margin: 8px 0;
  font-size: 12px;
}

.image-rating {
  padding: 2px 6px;
  border-radius: 4px;
  font-weight: bold;
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
  z-index: 1000;
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
.download {
  color: #2196f3;
  cursor: pointer;
  font-size: 12px;
}
</style>
