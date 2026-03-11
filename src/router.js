import { createWebHashHistory, createRouter } from 'vue-router'

import HomeView from './views/home.vue'
import InfoView from './views/info.vue'

const routes = [
  { path: '/', component: HomeView },
  { path: '/info', component: InfoView }
]

const router = createRouter({
  history: createWebHashHistory(),
  routes,
})

export default router