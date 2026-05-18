import { createWebHashHistory, createRouter } from 'vue-router'

import HomeView from './views/home.vue'
import YandeView from './views/yande.vue'
import KonachanView from './views/konachan.vue'
import DanbooruView from './views/danbooru.vue'
import GelbooruView from './views/gelbooru.vue'
import InfoView from './views/info.vue'

const routes = [
  { path: '/', component: HomeView },
  { path: '/yande', component: YandeView },
  { path: '/konachan', component: KonachanView },
  { path: '/danbooru', component: DanbooruView },
  { path: '/gelbooru', component: GelbooruView },
  { path: '/info', component: InfoView }
]

const router = createRouter({
  history: createWebHashHistory(),
  routes,
})

export default router