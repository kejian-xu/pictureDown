<script setup>
import { ref, computed, watch, onMounted, onUnmounted } from "vue";
import { invoke } from "@tauri-apps/api/core";

const props = defineProps({
  visible: {
    type: Boolean,
    required: true
  },
  images: {
    type: Array,
    required: true,
    default: () => []
  },
  selectedIndex: {
    type: Number,
    default: null
  }
});

const emit = defineEmits([
  'update:visible',
  'update:selectedIndex'
]);

const currentImageUrl = ref(null);
const loadingImage = ref(false);
const urlCache = ref({});

const selectedImage = computed(() => {
  if (props.selectedIndex === null || !props.images[props.selectedIndex]) {
    return null;
  }
  return props.images[props.selectedIndex];
});

const showModal = computed(() => props.visible);

async function loadImage(url) {
  if (urlCache.value[url]) {
    return urlCache.value[url];
  }

  try {
    const base64 = await invoke("fetch_image_as_base64", { url });
    let base64Img = `data:image/jpeg;base64,${base64}`;
    urlCache.value[url] = base64Img;
    return base64Img;
  } catch (error) {
    console.error("Failed to load image:", error);
  }
  return null;
}

async function loadCurrentImage() {
  currentImageUrl.value = null;
  const url = selectedImage.value?.sample_url;
  if (!url) {
    return;
  }

  loadingImage.value = true;
  const imgUrl = await loadImage(url);
  if (imgUrl) {
    currentImageUrl.value = imgUrl;
  } else {
    currentImageUrl.value = null;
  }
  loadingImage.value = false;
}

watch(() => props.selectedIndex, async (newIndex) => {
  if (newIndex === null) {
    currentImageUrl.value = null;
    loadingImage.value = false;
    return;
  }
  await loadCurrentImage();
}, { immediate: true });

function closeModal() {
  emit('update:selectedIndex', null);
  emit('update:visible', false);
}

function nextImage() {
  if (props.selectedIndex === null) return;
  const nextIndex = (props.selectedIndex + 1) % props.images.length;
  emit('update:selectedIndex', nextIndex);
}

function prevImage() {
  if (props.selectedIndex === null) return;
  const prevIndex = (props.selectedIndex - 1 + props.images.length) % props.images.length;
  emit('update:selectedIndex', prevIndex);
}

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

onMounted(() => {
  document.addEventListener("keydown", handleKeydown);
});

onUnmounted(() => {
  document.removeEventListener("keydown", handleKeydown);
  // 清理缓存
  urlCache.value = {};
});
</script>

<template>
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
            v-if="selectedImage"
            :src="selectedImage.src"
            :alt="selectedImage.alt"
            class="modal-image"
          />
        </div>
        <div v-else-if="currentImageUrl" class="modal-image-wrapper">
          <img
            :src="currentImageUrl"
            :alt="selectedImage?.alt"
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
</template>

<style scoped>
.image-modal {
  position: fixed;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  background-color: rgba(0, 0, 0, 0.8);
  display: flex;
  justify-content: center;
  align-items: center;
  z-index: 2000;
}

.modal-content {
  position: relative;
  max-width: 90%;
  max-height: 90%;
  background-color: #fff;
  border-radius: 8px;
  overflow: hidden;
  box-shadow: 0 4px 20px rgba(0, 0, 0, 0.3);
}

.modal-close {
  position: absolute;
  top: 10px;
  right: 15px;
  font-size: 30px;
  color: #fff;
  cursor: pointer;
  z-index: 1001;
  background-color: rgba(0, 0, 0, 0.5);
  width: 40px;
  height: 40px;
  border-radius: 50%;
  display: flex;
  justify-content: center;
  align-items: center;
  transition: background-color 0.3s;
}

.modal-close:hover {
  background-color: rgba(0, 0, 0, 0.8);
}

.modal-image-container {
  width: 100%;
  height: 100%;
  display: flex;
  justify-content: center;
  align-items: center;
  min-height: 300px;
}

.modal-loading {
  width: 100%;
  height: 100%;
  display: flex;
  justify-content: center;
  align-items: center;
}

.modal-image-wrapper {
  width: 100%;
  height: 100%;
  display: flex;
  justify-content: center;
  align-items: center;
  overflow: hidden;
}

.modal-image {
  max-width: 100%;
  max-height: 100%;
  object-fit: contain;
}

.modal-error {
  color: #f56c6c;
  font-size: 16px;
  padding: 40px;
  text-align: center;
}

.modal-navigation {
  position: absolute;
  top: 50%;
  left: 0;
  right: 0;
  transform: translateY(-50%);
  display: flex;
  justify-content: space-between;
  padding: 0 20px;
  pointer-events: none;
}

.nav-button {
  pointer-events: auto;
  font-size: 40px;
  color: #fff;
  cursor: pointer;
  background-color: rgba(0, 0, 0, 0.5);
  width: 50px;
  height: 50px;
  border-radius: 50%;
  display: flex;
  justify-content: center;
  align-items: center;
  transition: background-color 0.3s;
}

.nav-button:hover {
  background-color: rgba(0, 0, 0, 0.8);
}

.prev-button {
  margin-right: auto;
}

.next-button {
  margin-left: auto;
}
</style>