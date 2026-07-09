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
const protocolPrefix = site?.protocol_prefix || "https:";
const detailParseConfig = site?.detail?.parseConfig || {};
const detailUrlTemplate = site?.detail?.url || "";
const extractMethod = site?.detail?.extract_method || "css";
const hasPage = detailParseConfig.page === true;
const scriptMatchRegex = site?.detail?.script_match
  ? new RegExp(site.detail.script_match, "s")
  : null;
const imgUrlKey = site?.detail?.img_url_key || "page_url";

/** 统一URL补全：处理 // 协议相对、/绝对路径、相对路径 */
function resolveUrl(href, base) {
  if (href.startsWith("http")) return href;
  if (href.startsWith("//")) return protocolPrefix + href;
  if (href.startsWith("/")) return base + href;
  return `${base}/${href}`;
}

// ============ 响应式数据 ============

/** 图片列表（已加载的base64） */
const images = ref([]);

/** 所有原始URL（当前页解析后暂存，加载后清空） */
const allUrls = ref([]);

/** 已加载数量 */
const loadedCount = ref(0);

/** 每批加载数量 */
const BATCH_SIZE = 5;

/** 加载状态 */
const loading = ref(false);

/** 加载更多状态 */
const loadingMore = ref(false);

/** 错误信息 */
const error = ref("");

/** 是否还有更多（图片/页）待加载 */
const hasMore = ref(true);

/** ---- 分页模式专用 ---- */

/** 分页链接列表 */
const pageLinks = ref([]);

/** 当前子页码 */
const subPage = ref(1);

// ============ 生命周期 ============

onMounted(() => {
  if (!site) {
    error.value = `未找到站点配置: ${name}`;
    return;
  }
  const detailPath = route.query.detailPath;
  const directUrl = route.query.url;
  let url;
  if (detailPath && detailUrlTemplate) {
    url = mainUrl + detailUrlTemplate.replace("{detailPath}", detailPath);
  } else if (directUrl) {
    url = directUrl;
  }
  if (url) {
    fetchDetail(url);
  } else {
    error.value = "缺少详情页地址";
  }
});

onUnmounted(() => {
});

// ============ URL 提取 ============

/** CSS选择器方式提取图片URL */
function extractUrlsByCss($) {
  const urls = [];
  $(detailParseConfig.img_selector).each((_, el) => {
    let src = "";
    for (const attr of detailParseConfig.src_attrs) {
      const val = $(el).attr(attr);
      if (val && !val.startsWith("data:")) {
        src = val;
        break;
      }
    }
    if (src) urls.push(resolveUrl(src, mainUrl));
  });
  return urls;
}

/** script_json方式：从<script>标签中匹配JS数据，直接正则提取图片URL */
function extractUrlsByScriptJson($) {
  const urls = [];
  $("script").each((_, el) => {
    const text = $(el).html();
    if (!text || !text.includes(imgUrlKey)) return;
    const m = text.match(scriptMatchRegex);
    if (!m || !m[1]) return;
    const urlMatches = m[1].match(/"(https?:\/\/[^"]+)"/g);
    if (urlMatches) {
      for (const u of urlMatches) {
        urls.push(resolveUrl(u.slice(1, -1), mainUrl));
      }
    }
  });
  return urls;
}

/** string方式：从字符串直接正则提取图片URL */
function extractUrlsByString(str) {
  const urls = [];
  const urlMatches = str.match(/"(https?:\/\/[^"]+)"/g);
  if (urlMatches) {
    for (const u of urlMatches) {
      urls.push(resolveUrl(u.slice(1, -1), mainUrl));
    }
  }
  return urls;
}

/** 从HTML中提取分页链接 */
function extractPageLinks($) {
  const hrefs = [];
  $(detailParseConfig.page_size_selector).each((_, el) => {
    const href = $(el).attr("href");
    if (href && !hrefs.includes(href)) hrefs.push(href);
  });
  return hrefs;
}

// ============ 数据获取 ============

/** 解析HTML，提取图片URL和分页链接 */
function parseDetailHtml(html) {
  let urls = [];
  if (extractMethod === "string") {
    urls = extractUrlsByString(html);
  } else {
    const $ = cheerio.load(html);
    urls = extractMethod === "script_json"
      ? extractUrlsByScriptJson($)
      : extractUrlsByCss($);

    // 有分页时提取分页链接（仅首页提取一次）
    if (hasPage && pageLinks.value.length === 0) {
      pageLinks.value = extractPageLinks($);
    }
  }
  allUrls.value = urls;
  loadedCount.value = 0;
}

/** 加载下一批图片 */
async function loadNextBatch() {
  if (loadingMore.value || loadedCount.value >= allUrls.value.length) return;
  loadingMore.value = true;

  const batch = allUrls.value.slice(loadedCount.value, loadedCount.value + BATCH_SIZE);
  for (const url of batch) {
    try {
      const base64Url = await fetchImageBase64(url);
      images.value.push(base64Url || url);
    } catch (e) {
      console.error("获取图片失败:", url, e);
      images.value.push(url);
    }
  }
  loadedCount.value += batch.length;

  // 当前页还有图片 → 继续；否则 → 有分页就加载下一页，没有就结束
  const pageExhausted = loadedCount.value >= allUrls.value.length;
  if (pageExhausted && hasPage && subPage.value < pageLinks.value.length) {
    hasMore.value = true; // 下一页还在加载中，保持 true
  } else if (pageExhausted) {
    hasMore.value = false;
  } else {
    hasMore.value = true;
  }
  loadingMore.value = false;
}

/** 加载分页模式的下一页 */
async function loadNextPage() {
  if (loadingMore.value) return;
  const rawUrl = pageLinks.value[subPage.value];
  if (!rawUrl) {
    hasMore.value = false;
    return;
  }
  loadingMore.value = true;
  subPage.value++;
  try {
    const pageUrl = resolveUrl(rawUrl, mainUrl);
    const html = await invoke("fetch_html", { url: pageUrl });
    parseDetailHtml(html);
    await loadNextBatch();
  } catch (e) {
    console.error("加载下一页失败:", e);
    subPage.value--;
  } finally {
    loadingMore.value = false;
  }
}

/** 获取详情页（解析HTML + 加载首批图片） */
async function fetchDetail(url) {
  loading.value = true;
  error.value = "";
  try {
    const html = await invoke("fetch_html", { url });
    parseDetailHtml(html);
    await loadNextBatch();
  } finally {
    loading.value = false;
    loadingMore.value = false;
  }
}

/** 滚动加载更多：先尝试当前页的下一批，当前页耗尽时加载下一页 */
function loadMore() {
  if (loading.value || loadingMore.value) return;
  if (loadedCount.value < allUrls.value.length) {
    loadNextBatch();
  } else if (hasPage && subPage.value < pageLinks.value.length) {
    loadNextPage();
  }
}

async function fetchImageBase64(url) {
  if (url) {
    const base64 = await invoke("fetch_image_as_base64", { url });
    return `data:image/jpeg;base64,${base64}`;
  }
}

</script>

<template>
  <main>
    <div class="comic-section">
      <div class="comic-search-bar">
        <el-button icon="ArrowLeft" @click="$router.back()" circle />
      </div>

      <div v-if="error" class="comic-error">{{ error }}</div>

      <ul v-infinite-scroll="loadMore" :infinite-scroll-immediate="false" :infinite-scroll-distance="300" class="infinite-list">
        <li v-for="(img, i) in images" :key="i" class="infinite-list-item">
          <el-image :src="img" class="comic-image-box" />
        </li>
      </ul>
      <div v-if="loadingMore" class="loading text-center">
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
  width: 1200px;
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