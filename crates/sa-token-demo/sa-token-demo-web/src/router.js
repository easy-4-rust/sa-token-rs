// Vue Router 配置 —— 1:1 镜像 Java sa-token-demo-sso-client-vue3 的 router
import { createRouter, createWebHistory } from 'vue-router'
import LoginView from './views/LoginView.vue'
import HomeView from './views/HomeView.vue'
import UserView from './views/UserView.vue'

const routes = [
  { path: '/', redirect: '/home' },
  { path: '/login', name: 'login', component: LoginView },
  { path: '/home', name: 'home', component: HomeView, meta: { requiresAuth: true } },
  { path: '/user/list', name: 'user-list', component: UserView, meta: { requiresAuth: true, requiresPermission: 'user:list' } },
]

const router = createRouter({
  history: createWebHistory(),
  routes,
})

// 全局 beforeEach：从 cookie/localStorage 读 sa-token，未登录则跳 /login
router.beforeEach((to, _from, next) => {
  if (to.meta.requiresAuth) {
    const token = localStorage.getItem('satoken')
    if (!token) {
      return next({ path: '/login', query: { redirect: to.fullPath } })
    }
  }
  next()
})

export default router
