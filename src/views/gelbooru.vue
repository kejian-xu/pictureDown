<script setup>
import { ref } from "vue";
import ImageGallery from '../components/ImageGallery.vue';

const gelbooruUserId = ref("1976350");
const gelbooruApiKey = ref("e95dbb925076681754693f9d894fab6d5f49c26afe293065f912ea1e28d151253d06c41a7300aaeb0deaa54f724365c4e890277b4fe31945539dcb656cbc0a9d");
</script>

<template>
  <ImageGallery
    source="gelbooru.com"
    :authLogin="gelbooruUserId"
    :authApiKey="gelbooruApiKey"
  />
</template>
