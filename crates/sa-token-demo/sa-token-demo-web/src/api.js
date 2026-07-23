// Sa-Token 客户端 SDK —— 1:1 镜像 Java sa-token-sso-client-vue3 的 sa.js
import axios from 'axios'

const api = axios.create({
  baseURL: '/api',
  timeout: 5000,
})

// 请求拦截器：自动附加 sa-token
api.interceptors.request.use((config) => {
  const token = localStorage.getItem('satoken')
  if (token) {
    config.headers = config.headers || {}
    config.headers['satoken'] = token
  }
  return config
})

// 响应拦截器：处理 401 / 403
api.interceptors.response.use(
  (resp) => resp.data,
  (err) => {
    if (err.response?.status === 401) {
      localStorage.removeItem('satoken')
      window.location.href = '/login'
    } else if (err.response?.status === 403) {
      alert('权限不足：' + (err.response.data?.message || '请检查登录态'))
    }
    return Promise.reject(err)
  }
)

/** 登录 */
export function login(id, password) {
  return api.post('/login', { id, password })
}

/** 注销 */
export function logout() {
  return api.post('/logout')
}

/** 查询当前用户 */
export function getCurrentUser() {
  return api.get('/user/info')
}

/** 列出所有用户（需要 user:list 权限） */
export function listUsers() {
  return api.get('/user/list')
}

/** 检查 token 是否有效 */
export function checkLogin() {
  return api.get('/check-login')
}

export default api
