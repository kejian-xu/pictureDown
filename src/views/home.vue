<script setup>
import { useRouter } from 'vue-router';
// import siteConfig from './comic/comic.json';
const router = useRouter();

const sites = [
  {
    key: 'yande',
    name: 'Yande.re',
    description: '高分辨率动漫风格图片站',
    route: '/yande',
    color: '#409eff',
  },
  // {
  //   key: 'konachan',
  //   name: 'Konachan',
  //   description: '动漫壁纸图片站，支持标签搜索与批量下载',
  //   route: '/konachan',
  //   color: '#e6a23c',
  // },
  {
    key: 'danbooru',
    name: 'Danbooru',
    description: '全球最大的动漫图片社区，海量高质量作品',
    route: '/danbooru',
    color: '#f56c6c',
  },
  // {
  //   key: 'gelbooru',
  //   name: 'Gelbooru',
  //   description: '老牌图片站，资源丰富，无须特殊网络',
  //   route: '/gelbooru',
  //   color: '#67c23a',
  // },
  {
    key: 'rule34',
    name: 'Rule34',
    description: '知名图站，内容广泛不受限',
    route: '/rule34',
    color: '#9c27b0',
  },
  // 后续可以在此添加更多图站
];
const comic = [
  {
    key: '177comic',
    name: '177comic',
    description: '高质量漫画资源站',
    route: '/comic/home?name=177comic',
  },
  {
    key: 'wnacg',
    name: 'wnacg',
    description: '高质量漫画资源站',
    route: '/comic/home?name=wnacg',
  },
]
</script>

<template>
  <div class="home-page">
    <header class="home-header">
      <h1>PictureDown</h1>
      <p>多图站图片搜索与下载工具</p>
    </header>

    <div class="site-grid">
      <div
        v-for="site in sites"
        :key="site.key"
        class="site-card"
        @click="router.push(site.route)"
      >
        <div class="site-card-badge" :style="{ background: site.color }"></div>
        <div class="site-card-body">
          <h3>{{ site.name }}</h3>
          <p>{{ site.description }}</p>
        </div>
      </div>
    </div>

    <span class="home-footer">
      <span>更多图站即将接入…</span>
    </span>
    <div class="site-grid">
      <div
        v-for="site in comic"
        :key="site.key"
        class="site-card"
        @click="router.push(site.route)"
      >
        <div class="site-card-badge" ></div>
        <div class="site-card-body">
          <h3>{{ site.name }}</h3>
          <p>{{ site.description }}</p>
        </div>
      </div>
    </div>

  </div>
</template>

<style scoped>
.home-page {
  display: flex;
  flex-direction: column;
  align-items: center;
  padding: 0 20px;
}

.home-header {
  text-align: center;
  padding: 80px 0 48px;
}

.home-header h1 {
  font-size: 36px;
  font-weight: 700;
  color: #333;
  margin: 0 0 8px;
}

.home-header p {
  font-size: 15px;
  color: #888;
  margin: 0;
}

.site-grid {
  display: flex;
  flex-wrap: wrap;
  gap: 20px;
  max-width: 1200px;
  width: 100%;
  justify-content: flex-start;
}

.site-card {
  width: calc(25% - 20px);
  background: #16213e;
  border-radius: 10px;
  overflow: hidden;
  cursor: pointer;
  transition: transform 0.2s, box-shadow 0.2s;
  display: flex;
  flex-direction: column;
}

.site-card:hover {
  transform: translateY(-4px);
  box-shadow: 0 8px 24px rgba(0, 0, 0, 0.4);
}

.site-card-badge {
  height: 4px;
  width: 100%;
}

.site-card-body {
  padding: 24px 20px 20px;
  flex: 1;
}

.site-card-body h3 {
  font-size: 20px;
  font-weight: 600;
  color: #e0e0e0;
  margin: 0 0 8px;
}

.site-card-body p {
  font-size: 13px;
  color: #999;
  margin: 0;
  line-height: 1.6;
}

.site-card-arrow {
  padding: 0 20px 16px;
  color: #666;
  font-size: 16px;
  display: flex;
  justify-content: flex-end;
}

.home-footer {
  margin-top: auto;
  padding: 40px 0 24px;
  color: #555;
  font-size: 13px;
}
</style>
