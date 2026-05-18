<script setup>
import { ref } from "vue";
import ImageGallery from '../components/ImageGallery.vue';

const danbooruLogin = ref("xukejian11");
const danbooruApiKey = ref("QhhEGvNHZDetLijRFKpnPFgo");
</script>

<template>
  <ImageGallery
    source="danbooru.donmai.us"
    :authLogin="danbooruLogin"
    :authApiKey="danbooruApiKey"
  />
</template>
