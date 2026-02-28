<script setup>
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";

const tags = ref("");
const limit = ref(20);
const page = ref(1);
const images = ref([]);
const loading = ref(false);
const error = ref("");



async function fetchPosts() {
  loading.value = true;
  error.value = "";
  images.value = [];

  try {
    const posts = await invoke("fetch_posts", {
      tags: tags.value,
      limit: limit.value,
      page: page.value
    });

    // 将posts转换为images格式
    posts.forEach(post => {
      // 选择最佳图片URL：优先使用sample_url，然后file_url，最后preview_url
      let imageUrl = post.sample_url || post.file_url || post.preview_url;

      // 如果URL以https开头且遇到SSL问题，可以尝试转换为http
      // 注意：这可能会降低安全性，但可以解决某些证书问题
      // const fallbackUrl = imageUrl.replace('https://', 'http://');

      images.value.push({
        src: imageUrl,
        alt: post.tags,
        // 添加额外信息用于显示
        rating: post.rating,
        score: post.score,
        dimensions: `${post.width}x${post.height}`,
        // 保存备用URL用于错误回退
        fallbackUrl: post.file_url || post.preview_url || post.jpeg_url
      });
    });
  } catch (err) {
    error.value = err.message || String(err);
  } finally {
    loading.value = false;
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
  if (img.src && img.src.startsWith('https://')) {
    const httpUrl = img.src.replace('https://', 'http://');
    console.log(`Trying HTTP URL for image ${index}: ${httpUrl}`);
    event.target.src = httpUrl;
    return;
  }

  // 最后使用占位符图片
  event.target.src = 'https://via.placeholder.com/200x150?text=Image+Failed+to+Load';
}

fetchPosts();

</script>

<template>
  <main class="container">
    
  
    <div class="section">
      <h2>Yande.re Image Fetcher</h2>
      <form class="row" @submit.prevent="fetchPosts">
        <input id="tags-input" v-model="tags" placeholder="Enter tags (e.g., cute cat)" style="flex: 1; margin-right: 10px;" />
        <button type="submit" :disabled="loading">
          {{ loading ? 'Fetching...' : 'Fetch Images' }}
        </button>
      </form>
      <div class="row" style="margin-top: 10px;">
        <input type="number" v-model.number="limit" placeholder="Limit" style="width: 80px; margin-right: 10px;" min="1" max="100" />
        <input type="number" v-model.number="page" placeholder="Page" style="width: 80px;" min="1" />
      </div>
      
      <div v-if="error" class="error-message">
        {{ error }}
      </div>
      
      
      
      <div v-if="images.length > 0" class="images-container">
        <h3>Fetched Images ({{ images.length }}):</h3>
        <div class="images-grid">
          <div v-for="(img, index) in images" :key="index" class="image-item">
            {{ img.src }}
            <!-- <img :src="img.src" :alt="img.alt" @error="handleImageError($event, index)" class="extracted-image" /> -->
            <p class="image-info">{{ img.alt.length > 50 ? img.alt.substring(0, 50) + '...' : img.alt }}</p>
            <div class="image-meta">
              <span class="image-rating" :class="'rating-' + img.rating">{{ img.rating }}</span>
              <span class="image-score">Score: {{ img.score }}</span>
              <span class="image-dimensions">{{ img.dimensions }}</span>
            </div>
            <p class="image-src">{{ img.src }}</p>
          </div>
        </div>
      </div>
      
      <div v-if="images.length === 0 && !loading" class="no-images-message">
        No images found.
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
  padding-top: 10vh;
  display: flex;
  flex-direction: column;
  justify-content: center;
  text-align: center;
}

.row {
  display: flex;
  justify-content: center;
}

input,
button {
  border-radius: 8px;
  border: 1px solid transparent;
  padding: 0.6em 1.2em;
  font-size: 1em;
  font-weight: 500;
  font-family: inherit;
  color: #0f0f0f;
  background-color: #ffffff;
  transition: border-color 0.25s;
  box-shadow: 0 2px 2px rgba(0, 0, 0, 0.2);
}

button {
  cursor: pointer;
}

button:hover {
  border-color: #396cd8;
}

button:active {
  border-color: #396cd8;
  background-color: #e8e8e8;
}

input,
button {
  outline: none;
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
  margin-top: 30px;
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

  input,
  button {
    color: #ffffff;
    background-color: #0f0f0f98;
  }
  
  button:active {
    background-color: #0f0f0f69;
  }
}

</style>
