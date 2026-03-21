<script setup>
import { ref, computed, onUnmounted, watch, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { download } from "@tauri-apps/plugin-upload";
import { open } from "@tauri-apps/plugin-dialog";
import { downloadDir, join } from "@tauri-apps/api/path";
import { exists, remove } from "@tauri-apps/plugin-fs";
import { ElMessage } from 'element-plus'
import { appCacheDir } from '@tauri-apps/api/path';
import { debounce, throttle } from 'lodash-es'
import { MasonryGrid, MasonryGridItem } from 'vue3-masonry-css';
import { Check, Close, Clock } from '@element-plus/icons-vue'
import * as tauriCacheApi from 'tauri-plugin-cache-api';

// ============ 响应式数据 ============

/** 搜索标签 */
const tags = ref("");

/** 图片列表 */
const images = ref([]);

/** 加载状态 */
const loading = ref(false);

/** 错误信息 */
const error = ref("");

/** 当前选中的图片索引 */
const selectedImageIndex = ref(null);

/** 当前显示的图片URL */
const currentImageUrl = ref(null);

/** 图片加载状态 */
const loadingImage = ref(false);

/** URL缓存，用于存储图片blob URL */
const urlCache = ref({});

/** 当前页码 */
const currentPage = ref(1);

/** 每页显示数量 */
const pageSize = ref(50);

/** 总图片数量 */
const total = ref(0);

/** 下载目录路径 */
const downloadFilePath = ref("");

/** NSFW分级过滤 */
const nsfwModel = ref("rating:e");

/** 是否使用瀑布流布局 */
const isWaterfall = ref(false);

/** 选中的图片数组 */
const selectedImages = ref([]);

/** 设置对话框显示状态 */
const showSettings = ref(false);

/** 单个文件下载进度（0-100） */
const downloadProgress = ref(0);

/** 批量下载总进度（0-100） */
const batchDownloadProgress = ref(0);

/** 是否正在批量下载 */
const isBatchDownloading = ref(false);

/** 下载列表对话框显示状态 */
const showDownloadList = ref(false);

/** 下载列表，包含每个图片的下载状态 */
const downloadList = ref([]);

/** 当前选中的图片对象 */
const selectedImage = computed(() => {
  if (
    selectedImageIndex.value === null ||
    !images.value[selectedImageIndex.value]
  ) {
    return null;
  }
  return images.value[selectedImageIndex.value];
});

/** 模态框显示状态 */
const showModal = computed(() => selectedImageIndex.value !== null);

// ============ 生命周期钩子 ============

/**
 * 组件挂载时初始化
 * 加载下载目录设置并获取第一页图片
 */
/**
 * 组件挂载时初始化
 * 加载下载目录设置并获取第一页图片
 */
onMounted(async () => {
  fetchPosts();
  try {
    let defaultDownloadDir = await tauriCacheApi.get('downloadFilePath');
    console.log(defaultDownloadDir,'defaultDownloadDir')
    if (defaultDownloadDir && typeof defaultDownloadDir === 'string') {
      downloadFilePath.value = defaultDownloadDir;
      await invoke('grant_path_access', { path: defaultDownloadDir });
    } else {
      defaultDownloadDir = await downloadDir();
      downloadFilePath.value = defaultDownloadDir;
      await tauriCacheApi.set("downloadFilePath", defaultDownloadDir);
    }
  } catch (error) {
    console.error('初始化下载目录失败:', error);
    const defaultDownloadDir = await downloadDir();
    downloadFilePath.value = defaultDownloadDir;
    await tauriCacheApi.set("downloadFilePath", defaultDownloadDir);
  }

  // const appCachePath = await appCacheDir();
  // console.log("App Cache Directory:", appCachePath);
});

/**
 * 监听selectedImageIndex变化，加载对应图片
 */
watch(selectedImageIndex, async (newIndex) => {
  if (newIndex === null) {
    currentImageUrl.value = null;
    loadingImage.value = false;
    return;
  }
  await loadCurrentImage();
});

// ============ 工具函数 ============

/**
 * 滚动到指定元素
 * @param {string} id - 元素ID
 */
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

/**
 * 打开图片模态框
 * @param {number} index - 图片索引
 */
function openImageModal(index) {
  selectedImageIndex.value = index;
}

/**
 * 关闭图片模态框
 */
function closeModal() {
  selectedImageIndex.value = null;
}

/**
 * 显示下一张图片
 */
function nextImage() {
  if (selectedImageIndex.value === null) return;
  const nextIndex = (selectedImageIndex.value + 1) % images.value.length;
  selectedImageIndex.value = nextIndex;
}

/**
 * 显示上一张图片
 */
function prevImage() {
  if (selectedImageIndex.value === null) return;
  const prevIndex =
    (selectedImageIndex.value - 1 + images.value.length) % images.value.length;
  selectedImageIndex.value = prevIndex;
}

/**
 * 加载图片并返回blob URL
 * 支持内存缓存，避免重复加载
 * @param {string} url - 图片URL
 * @returns {Promise<string|null>} blob URL或null
 */
async function loadImage(url) {
  if (!url) return null;

  // 检查内存中的 blob URL 缓存
  if (urlCache.value[url]) {
    return urlCache.value[url];
  }


  // 从网络获取
  loadingImage.value = true;
  try {
    const base64 = await invoke("fetch_image_as_base64", { url });
    let base64Img = `data:image/jpeg;base64,${base64}`;
    urlCache.value[url] = base64Img;
    return  base64Img;
  } catch (err) {
    console.error('Failed to load image:', err);
    return null;
  } finally {
    loadingImage.value = false;
  }
}

/**
 * 加载当前选中的图片
 */
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
/**
 * 下载单个图片文件
 * 支持文件存在性检查和进度跟踪
 * @param {Object} img - 图片对象
 */
async function downloadFile(img) {
  img.loading = true; // 开始下载，设置 loading 状态
  img.downloadProgress = 0; // 初始化进度
  let picture = img.value || img; // 兼容直接传入图片对象或ref对象
  
  try {
    // 验证 downloadFilePath
    if (!downloadFilePath.value || typeof downloadFilePath.value !== 'string') {
      throw new Error('下载路径无效');
    }
    
    // 关键修复：await join 的返回值
    const savePath = await join(
      downloadFilePath.value,
      picture.md5 + "." + picture.file_ext
    );

    // 存在则跳过
    const existsPath = await exists(savePath);
    if (existsPath) {
      ElMessage.info(`文件已存在：${picture.md5}.${picture.file_ext}，已跳过`);
      return;
    }

    await download(
      picture.largeUrl, // 文件下载 URL
      savePath // 本地保存路径
    );
    ElMessage.success("下载成功！");
  } catch (error) {
    console.error("下载失败:", error);
    ElMessage.error("下载失败：" + error);
  } finally {
    img.loading = false; // 下载完成，重置 loading 状态
    img.downloadProgress = 0;
  }
}

/**
 * 批量下载选中的图片
 * 支持进度跟踪、错误处理和失败文件清理
 */
async function batchDownload() {
  if (selectedImages.value.length === 0) {
    ElMessage.warning("请先选择要下载的图片！");
    return;
  }

  // 验证下载路径
  if (!downloadFilePath.value || typeof downloadFilePath.value !== 'string') {
    ElMessage.error("下载路径无效，请先设置下载目录");
    return;
  }

  const total = selectedImages.value.length;
  
  // 初始化下载列表
  downloadList.value = selectedImages.value.map(img => ({
    ...img,
    status: 'pending', // pending, downloading, exists，completed, failed
    progress: 0,
    error: null
  }));

  isBatchDownloading.value = true;
  batchDownloadProgress.value = 0;
  showDownloadList.value = true; // 显示下载列表对话框
  
  ElMessage.info(`开始批量下载 ${total} 张图片...`);

  for (let index = 0; index < selectedImages.value.length; index++) {
    const img = selectedImages.value[index];
    const downloadItem = downloadList.value[index];

    const savePath = await join(
      downloadFilePath.value,
      img.md5 + "." + img.file_ext
    );

    downloadItem.status = 'downloading';
    downloadItem.progress = 0;

    try {
      // 文件已存在则跳过并标记完成
      let isExists = await exists(savePath)
      console.log(isExists,'isExists')
      if (isExists) {
        downloadItem.status = 'exists';
        downloadItem.progress = 100;
      } else {
        await download(img.largeUrl, savePath);
        downloadItem.status = 'completed';
        downloadItem.progress = 100;
      }
    } catch (error) {
      console.error("下载失败:", error);
      downloadItem.status = 'failed';
      downloadItem.error = error.message || String(error);
      downloadItem.progress = 0;

      // 删除失败文件（如果存在中间文件）
      try {
        if (await exists(savePath)) {
          await remove(savePath);
        }
      } catch (rmErr) {
        console.warn("删除失败文件失败:", rmErr);
      }
    }

    // 更新总体进度
    batchDownloadProgress.value = Math.round(((index + 1) / total) * 100);
  }

  isBatchDownloading.value = false;
  batchDownloadProgress.value = 100;

  const successCount = downloadList.value.filter(item => item.status === 'completed').length;
  const failCount = downloadList.value.filter(item => item.status === 'failed').length;

  if (failCount === 0) {
    ElMessage.success(`批量下载完成！成功下载 ${successCount} 张图片。`);
  } else {
    ElMessage.warning(`批量下载完成！成功 ${successCount} 张，失败 ${failCount} 张。`);
  }

  // 清空选择
  selectedImages.value = [];
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

// ============ 键盘事件处理 ============

/**
 * 处理键盘事件
 * 支持模态框中的导航和关闭操作
 * @param {KeyboardEvent} event - 键盘事件
 */
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

// ============ 事件监听器设置 ============

// 添加全局键盘事件监听
document.addEventListener("keydown", handleKeydown);

/**
 * 组件卸载时的清理工作
 * 移除事件监听器并清理blob URL防止内存泄漏
 */
onUnmounted(() => {
  document.removeEventListener("keydown", handleKeydown);
  // 撤销所有 blob URL 防止内存泄漏
  Object.values(urlCache.value).forEach(url => {
    URL.revokeObjectURL(url);
  });
  urlCache.value = {};
});

/**
 * 节流搜索处理函数
 * 重置页码并获取图片数据
 */
const handleSearch = throttle(() => {
  currentPage.value = 1
  fetchPosts();
},5000)
/**
 * 获取图片数据
 * 支持标签搜索、分页和NSFW过滤
 */
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
        return{
          ...post,
          src: imageUrl,
          alt: post.tags,
          created_at: formatTimestamp(post.created_at, "datetime"),
          dimensions: `${post.width}x${post.height}`,
          // 大图URL用于模态框显示
          largeUrl: post.file_url,
          loading: false,
          imageLoaded: false
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

/**
 * 格式化时间戳
 * @param {number} timestamp - Unix时间戳
 * @param {string} format - 格式类型：time/date/datetime/compact
 * @returns {string} 格式化后的时间字符串
 */
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

/**
 * 处理图片加载错误
 * 支持备用URL和HTTPS到HTTP的降级
 * @param {Event} event - 错误事件
 * @param {number} index - 图片索引
 */
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
}

/**
 * 处理图片加载成功
 * @param {number} index - 图片索引
 */
function handleImageLoad(index) {
  const img = images.value[index];
  if (img) {
    img.imageLoaded = true;
  }
}

/**
 * 处理标签选择
 * @param {string} tag - 选择的标签
 */
function handleTags(tag) {
  tags.value = tag;
  fetchPosts();
}


/**
 * 处理页面大小变化
 * @param {number} val - 新的页面大小
 */
const handleSizeChange = (val) => {
  pageSize.value = val;
  fetchPosts();
};

/**
 * 处理当前页变化
 * @param {number} val - 新的当前页
 */
const handleCurrentChange = (val) => {
  currentPage.value = val;
  fetchPosts();
};

/**
 * 瀑布流加载更多图片
 * 节流处理，避免频繁请求
 * @param {string} direction - 滚动方向
 */
const waterfallLoad = throttle((direction) =>{
  console.log('direction',direction)
  if(direction === 'bottom') {
    currentPage.value += 1;
    fetchPosts();
  }
},10000)

/**
 * 打开设置对话框
 */
function handleSetting() {
  showSettings.value = true;
}

/**
 * 选择下载目录
 * 使用系统文件对话框选择目录并保存设置
 */
async function selectDownloadPath() {
  try {
    const selected = await open({
      title: "选择下载目录",
      directory: true,
      defaultPath: downloadFilePath.value
    });
    
    if (selected && typeof selected === 'string') {
      downloadFilePath.value = selected;
      await tauriCacheApi.set('downloadFilePath', selected);
      ElMessage.success("下载路径已更新！");
      await invoke('grant_path_access', { path: selected });
    } else if (selected) {
      console.error('选择路径类型错误:', typeof selected);
      ElMessage.error("选择的路径格式错误");
    }
  } catch (error) {
    console.error('选择下载目录失败:', error);
    ElMessage.error("选择下载目录失败");
  }
}

/**
 * 保存设置
 */
function saveSettings() {
  // 设置在选择目录时已经保存，这里只是关闭对话框
  showSettings.value = false;
}

/**
 * 关闭下载列表对话框
 */
function closeDownloadList() {
  showDownloadList.value = false;
  // 重置下载列表
  downloadList.value = [];
}

/**
 * 切换图片选择状态
 * @param {Object} img - 图片对象
 */
function toggleImageSelection(img) {
  const index = selectedImages.value.findIndex(selected => selected.md5 === img.md5);
  if (index > -1) {
    selectedImages.value.splice(index, 1);
  } else {
    selectedImages.value.push(img);
  }
}

/**
 * 全选/取消全选图片
 */
function selectAllImages() {
  if (selectedImages.value.length === images.value.length) {
    selectedImages.value = [];
  } else {
    selectedImages.value = [...images.value];
  }
}


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
            <el-button @click="selectAllImages" type="warning">
              {{ selectedImages.length === images.length ? "取消全选" : "全选" }}
            </el-button>
            <div style="display: inline-block; margin-left: 10px;">
              <el-button 
                v-if="!isBatchDownloading" 
                @click="batchDownload" 
                type="success" 
                :disabled="selectedImages.length === 0"
              >
                批量下载 ({{ selectedImages.length }})
              </el-button>
            </div>
            <!-- <el-switch v-model="nsfwModel"></el-switch> -->
          </el-form-item>
          <div class="float-right">
             <el-button 
                @click="showDownloadList = true"
                type="primary"
              >
                查看下载进度
              </el-button>
            <el-button icon="Setting" @click="handleSetting">设置</el-button>
          </div>
        </el-form>
      </div>
    

      <div v-if="error" class="error-message">
        {{ error }}
      </div>

      <div  class="images-container" v-loading="loading" v-if="!isWaterfall">
        <div class="images-grid"  ref="masonryGrid">
          <div v-for="(img, index) in images" :key="index" class="image-item">
            <!-- {{ img.src }} -->
            <div class="image-container">
              <div class="image-checkbox">
                <el-checkbox
                  size="large"
                  :model-value="selectedImages.some(selected => selected.md5 === img.md5)" 
                  @change="toggleImageSelection(img)"
                ></el-checkbox>
              </div>
              <div v-if="!img.imageLoaded" class="image-placeholder">
                <!-- 占位符内容，可以是一个加载动画或灰色方块 -->
                <div class="placeholder-content">Loading...</div>
              </div>
              <img
                v-show="img.imageLoaded"
                :src="img.src"
                :alt="img.alt"
                class="extracted-image"
                @error="handleImageError($event, index)"
                @load="handleImageLoad(index)"
                @click="openImageModal(index)"
                style="cursor: pointer"
              />
            </div>
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
              <el-button :loading="img.loading" type="primary" round icon="Download" @click="downloadFile(img)"></el-button>
            </div>
          </div>
        </div>
      </div>
       <el-scrollbar 
        v-else
        height="calc(100vh - 85px)"
        always
        :distance="10"
        @end-reached="waterfallLoad"
      >
      <div  class="images-container waterfall" >
        <MasonryGrid 
            :columns="{ default: 8, 1024: 5, 768: 4, 480: 2}"
            :gutter="5"
          >
           <MasonryGridItem v-for="(img, index) in images" :key="index">
          <div  class="image-item" :id="img.md5">
            <div class="image-container">
              <div class="image-checkbox">
                <el-checkbox 
                  size="large"
                  :model-value="selectedImages.some(selected => selected.md5 === img.md5)" 
                  @change="toggleImageSelection(img)"
                ></el-checkbox>
              </div>
              <div v-if="!img.imageLoaded" class="image-placeholder">
                <div class="placeholder-content">Loading...</div>
              </div>
              <img
                v-show="img.imageLoaded"
                :src="img.src"
                :alt="img.alt"
                class="extracted-image"
                @error="handleImageError($event, index)"
                @load="handleImageLoad(index)"
                @click="openImageModal(index)"
                style="cursor: pointer"
              />
            </div>
            <div class="image-download" >
              <el-button :loading="img.loading" type="primary" round icon="Download" @click="downloadFile(img)"></el-button>
            </div>
          </div>
          </MasonryGridItem>
          </MasonryGrid>
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

    <!-- 设置对话框 -->
    <el-dialog v-model="showSettings" title="设置" width="500px">
      <el-form label-width="120px" class="settings-form">
        <el-form-item label="下载目录">
          <el-input 
            v-model="downloadFilePath" 
            readonly 
            placeholder="选择下载目录"
            style="width: 70%;"
          />
          <el-button @click="selectDownloadPath" type="primary" style="margin-left: 10px;">
            选择目录
          </el-button>
        </el-form-item>
        <el-form-item label="当前路径">
          <span style="color: #666; font-size: 12px;">{{ downloadFilePath }}</span>
        </el-form-item>
      </el-form>
      <template #footer>
        <span class="dialog-footer">
          <el-button @click="showSettings = false">取消</el-button>
          <el-button type="primary" @click="saveSettings">保存</el-button>
        </span>
      </template>
    </el-dialog>

    <!-- 下载列表对话框 -->
    <el-dialog v-model="showDownloadList" title="下载进度" width="800px" :close-on-click-modal="false" >
      <div class="download-list-container">
        <div class="download-items" v-if="downloadList.length > 0">
          <div 
            v-for="(item, index) in downloadList" 
            :key="item.md5" 
            class="download-item"
            :class="`status-${item.status}`"
          >
            <div class="download-item-image">
              <img :src="item.src" :alt="item.alt" />
            </div>
            
            <div class="download-item-info">
              <div class="download-item-name">
                {{ item.md5 }}.{{ item.file_ext }}
              </div>
              <div class="download-item-size">
                {{ item.dimensions }}
              </div>
              <div class="download-item-status">
                <span v-if="item.status === 'pending'" class="status-text">等待中</span>
                <span v-else-if="item.status === 'downloading'" class="status-text downloading">
                  下载中... {{ item.progress }}%
                </span>
                <span v-else-if="item.status === 'exists'" class="status-text success">文件已存在</span>
                <span v-else-if="item.status === 'completed'" class="status-text success">下载完成</span>
                <span v-else-if="item.status === 'failed'" class="status-text error">下载失败</span>
              </div>
              <div v-if="item.error" class="download-item-error">
                {{ item.error }}
              </div>
            </div>
            
            <div class="download-item-progress">
              <el-progress 
                v-if="item.status === 'downloading'"
                type="circle" 
                :percentage="item.progress" 
                :width="40" 
                :stroke-width="3"
              ></el-progress>
              <el-icon v-else-if="item.status === 'completed' || item.status === 'exists'" size="24" color="#67C23A">
                <Check />
              </el-icon>
              <el-icon v-else-if="item.status === 'failed'" size="24" color="#F56C6C">
                <Close />
              </el-icon>
              <el-icon v-else size="24" color="#909399">
                <Clock />
              </el-icon>
            </div>
          </div>
        </div>
        <div v-else class="text-center">
          当前无下载任务
        </div>
      </div>
      
      <template #footer>
        <span class="dialog-footer">
          <el-button @click="closeDownloadList" :disabled="isBatchDownloading">
            {{ isBatchDownloading ? '下载中...' : '关闭' }}
          </el-button>
        </span>
      </template>
    </el-dialog>
  </main>
</template>

<style>



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
  grid-template-rows: masonry; 
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

.image-container {
  position: relative;
  width: 100%;
  height: 200px; /* 与 .extracted-image 高度一致 */
  display: flex;
  align-items: center;
  justify-content: center;
}

.image-placeholder {
  position: absolute;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  background-color: #f5f5f5;
  border-radius: 4px;
  display: flex;
  align-items: center;
  justify-content: center;
  color: #999;
  font-size: 14px;
}

.waterfall .image-container {
  height: auto; /* 瀑布流中高度自适应 */
  min-height: 100px; /* 最小高度 */
}

.waterfall .image-placeholder {
  position: relative;
  padding-bottom: 75%; /* 4:3 宽高比占位 */
  height: 0;
}

.waterfall .placeholder-content {
  position: absolute;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
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

.download-progress {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 50px;
  height: 50px;
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

/* 设置对话框样式 */
.settings-form .el-form-item {
  margin-bottom: 20px;
}

.settings-form .el-form-item__label {
  font-weight: 500;
}

.dialog-footer {
  text-align: right;
}

/* 下载列表对话框样式 */
.download-list-container {
  max-height: 500px;
}

.download-summary {
  margin-bottom: 20px;
}

.download-items {
  max-height: 400px;
  overflow-y: auto;
}

.download-item {
  display: flex;
  align-items: center;
  padding: 12px;
  border: 1px solid #e4e7ed;
  border-radius: 6px;
  margin-bottom: 8px;
  background-color: #fafafa;
  transition: all 0.2s;
}

.download-item:hover {
  background-color: #f5f5f5;
}

.download-item.status-downloading {
  border-color: #409eff;
  background-color: #ecf5ff;
}

.download-item.status-completed {
  border-color: #67c23a;
  background-color: #f0f9ff;
}

.download-item.status-failed {
  border-color: #f56c6c;
  background-color: #fef0f0;
}

.download-item-image {
  width: 60px;
  height: 60px;
  margin-right: 12px;
  border-radius: 4px;
  overflow: hidden;
  flex-shrink: 0;
}

.download-item-image img {
  width: 100%;
  height: 100%;
  object-fit: cover;
}

.download-item-info {
  flex: 1;
  min-width: 0;
}

.download-item-name {
  font-weight: 500;
  color: #303133;
  margin-bottom: 4px;
  word-break: break-all;
}

.download-item-size {
  font-size: 12px;
  color: #909399;
  margin-bottom: 4px;
}

.download-item-status {
  font-size: 12px;
}

.status-text {
  font-weight: 500;
}

.status-text.downloading {
  color: #409eff;
}

.status-text.success {
  color: #67c23a;
}

.status-text.error {
  color: #f56c6c;
}

.download-item-error {
  font-size: 11px;
  color: #f56c6c;
  margin-top: 4px;
  word-break: break-all;
}

.download-item-progress {
  margin-left: 12px;
  display: flex;
  align-items: center;
  justify-content: center;
  width: 40px;
  flex-shrink: 0;
}
</style>
