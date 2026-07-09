<script setup>
import { ref, onMounted, onUnmounted } from "vue";
import { useRoute } from 'vue-router';
import { invoke } from "@tauri-apps/api/core";
import { ElMessage } from 'element-plus'
import * as cheerio from 'cheerio';
import siteConfig from './comic.json';

const route = useRoute();

// ============ 站点配置 ============

/** 从 comic.json 中根据 name 获取当前站点detail配置 */
const name = route.query.name || "177comic";
const site = siteConfig[name];
const mainUrl = site?.main_url || "";
const detailParseConfig = site?.detail?.parseConfig || {};
const detailUrlTemplate = site?.detail?.url || "";

// ============ 响应式数据 ============

/** 图片列表 */
const images = ref([]);

/** 加载状态 */
const loading = ref(false);

/** 加载更多状态 */
const loadingMore = ref(false);

/** 错误信息 */
const error = ref("");

/** 总数量 */
const total = ref(0);

/** 详情页 URL */
const detailUrl = ref("");

/** 当前子页码 */
const subPage = ref(1);

/** 是否还有更多页 */
const hasMore = ref(true);

/** 总页数（从 page-links 解析） */
const totalPages = ref(0);
const pages = ref([])

// ============ 生命周期 ============

let scrollHandler = null;



onMounted(() => {
  const url = route.query.url;
  if (!site) {
    error.value = `未找到站点配置: ${name}`;
    return;
  }
  if (url) {
    detailUrl.value = url;
    fetchDetail(url,1)
  } else {
    error.value = "缺少详情页地址";
  }
});

onUnmounted(() => {
});

// ============ 数据获取 ============


/** 前端解析HTML，提取详情页图片URL列表，并转换为base64 */
async function parseDetailHtml(html) {
  const $ = cheerio.load(html);
  const urls = [];

  $(detailParseConfig.img_selector).each((index, element) => {
    let src = "";
    for (const attr of detailParseConfig.src_attrs) {
      const val = $(element).attr(attr);
      if (val && !val.startsWith("data:")) {
        src = val;
        break;
      }
    }
    if (src) {
      // 相对URL补全为绝对URL
      urls.push(src.startsWith("http") ? src : mainUrl + src);
    }
  });
  // 从 page-links 的 a 标签中提取总页数
  const hrefs = [];
  if(pages.value.length == 0) {
    $(detailParseConfig.page_size_selector).each((i, el) => {
      let href = $(el).attr("href")
      if(href && !hrefs.includes(href)) {
        hrefs.push(href);
      }
    });
    pages.value = hrefs
    totalPages.value = hrefs.length
  }
  

  // 逐个获取图片的 base64 数据
  for (const url of urls) {
    try {
      const base64Url = await fetchImageBase64(url);
      if (base64Url) {
        images.value.push(base64Url);
      }
    } catch (e) {
      console.error("获取图片失败:", url, e);
      // 失败时也保留原 URL，让 el-image 尝试直接加载
      images.value.push(url);
    }
  }
}


/** 获取详情页图片 */
async function fetchDetail(url, page) {
  if (page === 1) {
    loading.value = true;
    error.value = "";
  }

  try {
    const pageUrl = url;
    const html = await invoke("fetch_html", { url: pageUrl });
    await parseDetailHtml(html);
    // 判断是否还有下一页
    hasMore.value = subPage.value < totalPages.value;
  } finally {
    loading.value = false;
    loadingMore.value = false;
  }
}
function loadMore() {
  console.log('loadMore')
  if (loading.value || loadingMore.value || !hasMore.value) return;
  loadNextPage();
}

/** 加载下一页 */
async function loadNextPage() {
  if (loading.value || loadingMore.value || !hasMore.value) return;
  loadingMore.value = true;
  subPage.value++;
  const rawUrl = pages.value[subPage.value - 1];
  if (!rawUrl) {
    hasMore.value = false;
    loadingMore.value = false;
    return;
  }
  const pageUrl = rawUrl.startsWith("http") ? rawUrl : mainUrl + rawUrl;
  try {
    console.log('pageUrl',pageUrl,subPage.value)
    await fetchDetail(pageUrl, subPage.value);
  } catch (e) {
    console.error("加载下一页失败:", e);
    subPage.value--;
  } finally {
    loadingMore.value = false;
  }
}

async function fetchImageBase64(url) {
  console.log("url:", url);
  if(url) {
    let base64 =  await invoke("fetch_image_as_base64", { url })
    return `data:image/jpeg;base64,${base64}`;
  }
}

</script>

<template>
  <main>
    <div class="comic-section">
      <!-- 搜索栏 -->
      <div class="comic-search-bar">
        <el-button icon="ArrowLeft" @click="$router.back()" circle />
      </div>

      <!-- 错误信息 -->
      <div v-if="error" class="comic-error">{{ error }}</div>

    <ul v-infinite-scroll="loadMore" :infinite-scroll-immediate="false" :infinite-scroll-distance="300" class="infinite-list">
      <li v-for="(img, i) in images" :key="i" class="infinite-list-item">
        <el-image :src="img" />
      </li>
    </ul>
    <div v-if="loadingMore">
      加载中。。。
    </div>
    </div>
  </main>
</template>

<style scoped>
.comic-section {
  width: 100%;
  margin: 0 auto;
}
.infinite-list-item {
  text-align: center;
  list-style: none;
}

.comic-count {
  color: #666;
  font-size: 14px;
}

.comic-search-bar {
  position: sticky;
  top: 0;
  background-color: white;
  padding: 10px;
  z-index: 1001;
  box-shadow: 0 2px 10px rgba(0, 0, 0, 0.1);
  border-radius: 0 0 8px 8px;
}

.comic-error {
  color: #e53935;
  margin: 10px 20px;
  padding: 10px;
  background-color: rgba(229, 57, 53, 0.1);
  border-radius: 4px;
}

.comic-images-wrap {
  margin: 10px 0;
  padding: 0 10px;
  min-height: 200px;
}

.comic-images-list {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(800px, 1fr));
  gap: 0;
}

.comic-image-cell {
  position: relative;
}

.comic-image-box {
  width: 800px;
  margin: 0 auto;
}

.comic-image {
  width: 100%;
  height: auto;
  display: block;
}

.comic-empty {
  margin-top: 20px;
  padding: 15px;
  background-color: #f5f5f5;
  border-radius: 4px;
  color: #666;
  text-align: center;
}

.comic-loading-more {
  height: 40px;
  display: flex;
  align-items: center;
  justify-content: center;
  color: #999;
  font-size: 13px;
}
</style>
