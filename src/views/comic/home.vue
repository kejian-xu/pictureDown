<script setup>
import { ref, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { ElMessage } from 'element-plus'
import { useRouter, useRoute } from 'vue-router';
import * as cheerio from 'cheerio';
import siteConfig from './comic.json';

const router = useRouter();
const route = useRoute();

// ============ 站点配置 ============

/** 从 comic.json 中根据 name 获取当前站点配置 */
const name = route.query.name;
const site = siteConfig[name];
const mainUrl = site?.main_url || "";

const protocolPrefix = site?.protocol_prefix || "https:";
const parseConfig = site?.home?.parseConfig || {};
const detailPathRegex = site?.home?.detail_path_regex
  ? new RegExp(site.home.detail_path_regex)
  : null;
const catgory = site?.home?.catgory || []

const hasPage = parseConfig.page === true;

/** 统一URL补全：处理 // 协议相对、/绝对路径、相对路径 */
function resolveUrl(href, base) {
  if (href.startsWith("http")) return href;
  if (href.startsWith("//")) return protocolPrefix + href;
  if (href.startsWith("/")) return base + href;
  return `${base}/${href}`;
}

// ============ 响应式数据 ============

/** 请求地址 */
const parentUrl = ref("")
const homeUrl = ref("")
const activeUrl= ref("")
/** 图片列表 */
const images = ref([]);

/** 加载状态 */
const loading = ref(false);

/** 错误信息 */
const error = ref("");

const pageLinks = ref([]);
const currentPage = ref(1);
const pageSize = ref(10);
const maxPage = ref(0);
const pageUrlPattern = parseConfig.page_size_url || "";

// ============ HTML 解析 ============

/**
 * 使用 cheerio 解析漫画站HTML，提取图片信息
 * 通过 config 中的 CSS 选择器配置适配不同的漫画站
 */
function parseComicHtml(html, baseUrl, config) {
  const $ = cheerio.load(html);
  const base = baseUrl;
  const cfg = config || {};

  const posts = [];

  $(cfg.item_selector).each((_, item) => {
    $(item).find(cfg.link_selector || "a").each((_, a) => {
      const $a = $(a);
      const $img = $a.find(cfg.img_selector || "img").first();
      if (!$img.length) return;

      const href = $a.attr(cfg.link_attr || "href");
      if (!href) return;

      const src = $img.attr(cfg.thumb_attr || "src");
      if (!src) return;

      const title = $img.attr(cfg.title_attr || "alt") || "";

      const postUrl = resolveUrl(href, base);
      const thumbUrl = resolveUrl(src, base);

      // 从 post_url 的相对路径中提取 detailPath
      let detailPath = "";
      if (detailPathRegex) {
        const path = postUrl.replace(mainUrl, "");
        const m = path.match(detailPathRegex);
        if (m) detailPath = m[1];
      }

      posts.push({
        thumb_url: thumbUrl,
        post_url: postUrl,
        title,
        detailPath,
      });
    });
  });
 // 有分页时提取最大页码并自动生成分页链接（仅在页面初始化时执行一次）
  if (hasPage && pageLinks.value.length === 0) {
    let max = 1;
    $(cfg.page_size_selector).each((_, el) => {
      const href = $(el).attr("href");
      if (!href) return;
      // 从 page_url_pattern 匹配页码，例如 /page/{num}/ → 匹配 /page/1662/
      const escaped = pageUrlPattern.replace(/\{[^}]+\}/g, "(\\d+)");
      const m = href.match(new RegExp(escaped));
      if (m) {
        const n = parseInt(m[1], 10);
        if (n > max) max = n;
      }
    });
    maxPage.value = max;
    // 自动生成页码数组
    for (let i = 1; i <= max; i++) {
      pageLinks.value.push({
        num: i,
        url: pageUrlPattern.replace(/\{[^}]+\}/g, i),
      });
    }
  }

  return { posts, count: posts.length };
}

// ============ 生命周期 ============

onMounted(() => {
  if (!site) {
    error.value = `未找到站点配置: ${name}`;
    return;
  }
  parentUrl.value =  mainUrl + site?.home?.url;
  homeUrl.value = mainUrl + (site?.home?.url || "/");
  fetchData();
});

// ============ 数据获取 ============

async function fetchData() {
  loading.value = true;
  error.value = "";

  try {
    console.log("homeUrl",homeUrl.value)
    const html = await invoke("fetch_html", { url: homeUrl.value });
    const result = parseComicHtml(html, mainUrl, parseConfig);

    images.value = result.posts.map((post, i) => ({
      ...post,
      src: post.thumb_url,
      alt: post.title,
      md5: post.post_url || `${i}`,
    }));

    if (result.posts.length > 0) {
      ElMessage.success(`加载成功，共 ${result.count} 条`);
    } else {
      ElMessage.info("当前页没有图片数据");
    }
  } catch (err) {
    error.value = err.message || String(err);
    ElMessage.error("加载失败：" + error.value);
  } finally {
    loading.value = false;
  }
}
function handleChangePage(item) {
  activeUrl.value = item?.url;
  currentPage.value = 1;
  parentUrl.value =  mainUrl + item?.url;
  homeUrl.value = mainUrl + (item?.url || "/");
  pageLinks.value = [];
  maxPage.value = 0;
  fetchData();
}
function goToPage(page) {
  if (page === currentPage.value) return;
  currentPage.value = page;
  homeUrl.value = parentUrl.value + pageLinks.value[page - 1].url;
  fetchData();
}



// ============ 图片交互 ============

function handleImageClick(img) {
  if (img.detailPath) {
    router.push({ path: '/comic/detail', query: { detailPath: img.detailPath, name } });
  } else if (img.post_url) {
    router.push({ path: '/comic/detail', query: { url: img.post_url, name } });
  }
}

function handleImageError(event, index) {
  const img = images.value[index];
  if (!img) return;

  img._loadAttempt = (img._loadAttempt || 0) + 1;

  if (img._loadAttempt === 1 && img.src?.startsWith("https://")) {
    event.target.src = img.src.replace("https://", "http://");
    return;
  }

  images.value.splice(index, 1);
}
</script>

<template>
  <main >
    <div class="comic-section">
      <!-- 顶栏 -->
      <div class="comic-search-bar">
        <div class="comic-search-bar-left">
          <el-button icon="ArrowLeft" @click="$router.push('/')" circle />
          <span v-for="item in catgory" @click="handleChangePage(item)" :class="activeUrl === item.url? 'active':''">
            {{ item.title }}
          </span>
        </div>
        <!-- <el-button class="" type="primary" @click="fetchData" :loading="loading">
          {{ loading ? "加载中..." : "刷新" }}
        </el-button> -->
      </div>

      <!-- 错误信息 -->
      <div v-if="error" class="comic-error">{{ error }}</div>

      <!-- 图片网格 -->
      <div class="comic-list-wrap" v-loading="loading">
        <div class="comic-list-grid" v-if="images.length > 0">
          <div v-for="(img, index) in images" :key="img.md5" class="comic-card" @click="handleImageClick(img)">
            <div class="comic-card-img">
              <el-image
                :src="img.src"
                lazy
                class="comic-thumb"
                @error="handleImageError($event, index)"
                fit="contain"
                show-progress
              />
            </div>
            <div class="comic-card-meta">
              <span class="comic-card-title">{{ img.title }}</span>
            </div>
          </div>
        </div>

        <div v-if="images.length === 0 && !loading" class="comic-empty">
          暂无图片数据
        </div>
      </div>
      <!-- 分页 -->
      <div class="comic-pagination" v-if="hasPage && maxPage > 1">
          <el-pagination
          :current-page="currentPage"
          background
          prev-text="上一页"
          next-text="下一页"
          layout="total, prev, pager, next, jumper"
          :total="maxPage * 10"
          @current-change="goToPage"
        />
      </div>
    </div>
  </main>
</template>

<style scoped>
.comic-section {
  width: 100%;
  margin: 0 auto;

}

.comic-search-bar {
  position: sticky;
  top: 0;
  background-color: white;
  padding: 10px;
  z-index: 1001;
  box-shadow: 0 2px 10px rgba(0, 0, 0, 0.1);
  border-radius: 0 0 8px 8px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 10px;
}
.comic-search-bar-left {
  display: flex;
  align-items: center;
  gap: 10px;
}
.comic-search-bar-left span{
  cursor: pointer;
}
.comic-search-bar-left span.active {
  color: var(--el-color-primary);
  border-bottom: solid 1px var(--el-color-primary);
}


.comic-error {
  color: #e53935;
  margin-top: 10px;
  padding: 10px;
  background-color: rgba(229, 57, 53, 0.1);
  border-radius: 4px;
}

.comic-list-wrap {
  margin: 10px 0;
  padding: 0 10px;
  min-height: 200px;
}

.comic-list-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
  gap: 20px;
  margin-top: 20px;
}

.comic-card {
  border: 1px solid #e0e0e0;
  border-radius: 8px;
  padding: 5px;
  background-color: white;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.05);
  transition: transform 0.2s, box-shadow 0.2s;
  position: relative;
  cursor: pointer;
}

.comic-card:hover {
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
}

.comic-card-img {
  position: relative;
  width: 100%;
  height: 210px;
  display: flex;
  align-items: center;
  justify-content: center;
}

.comic-thumb {
  max-width: 100%;
  height: 210px;
  border-radius: 4px;
  display: block;
  margin: 0 auto;
}

.comic-card-meta {
  padding: 8px 5px 0;
}

.comic-card-title {
  font-size: 12px;
  color: #666;
  display: -webkit-box;
  line-clamp: 2;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
  overflow: hidden;
  line-height: 1.4;
}

.comic-empty {
  margin-top: 20px;
  padding: 15px;
  background-color: #f5f5f5;
  border-radius: 4px;
  color: #666;
  text-align: center;
}

.comic-pagination {
  position: sticky;
  bottom: 0;
  background-color: white;
  padding: 10px;
  box-shadow: 0 -2px 10px rgba(0, 0, 0, 0.1);
  z-index: 1001;
  display: flex;
  justify-content: center;
  gap: 10px;

}
.comic-pagination span {
  border-radius: 4px;
  padding: 0 5px;
  height: 20px;
  color: var(--el-color-primary);
  border: solid 1px var(--el-color-primary);
  cursor: pointer;
}
</style>
