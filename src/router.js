import { createWebHashHistory, createRouter } from 'vue-router'

import HomeView from './views/home.vue'
import YandeView from './views/yande.vue'
import KonachanView from './views/konachan.vue'
import DanbooruView from './views/danbooru.vue'
import GelbooruView from './views/gelbooru.vue'
import Rule34View from './views/rule34.vue'
import InfoView from './views/info.vue'
import ComicHomeView from './views/comic/home.vue'
import ComicDetailView from './views/comic/detail.vue'

const routes = [
  { path: '/', component: HomeView },
  { path: '/yande', component: YandeView },
  { path: '/konachan', component: KonachanView },
  { path: '/danbooru', component: DanbooruView },
  { path: '/gelbooru', component: GelbooruView },
  { path: '/rule34', component: Rule34View },
  { path: '/info', component: InfoView },
  { path: '/comic/home', component: ComicHomeView },
  { path: '/comic/detail', component: ComicDetailView }
]

const router = createRouter({
  history: createWebHashHistory(),
  routes,
})

export default router