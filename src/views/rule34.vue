<script setup>
import { ref } from "vue";
import ImageGallery from '../components/ImageGallery.vue';

const rule34UserId = ref("6263987");
const rule34ApiKey = ref("7d13c4072208e7c9a90c7afbd202d049dd353cce1779ccfcf40ccb1a37fe56c3e097f09be15080276623b77c304725b4b4eff5bf3d09b9749308cdf8bf9f2749");
</script>

<template>
  <ImageGallery
    source="rule34.xxx"
    :authLogin="rule34UserId"
    :authApiKey="rule34ApiKey"
  />
</template>
