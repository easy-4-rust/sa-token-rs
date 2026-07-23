<script setup>
import { ref, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { getCurrentUser, logout } from '../api.js'

const user = ref(null)
const error = ref('')
const router = useRouter()

onMounted(async () => {
  try {
    const resp = await getCurrentUser()
    user.value = resp
  } catch (err) {
    error.value = err.response?.data?.message || err.message
  }
})

async function onLogout() {
  try {
    await logout()
  } catch (_) {
    // 忽略后端报错，前端清理 token 即可
  }
  localStorage.removeItem('satoken')
  router.push('/login')
}
</script>

<template>
  <div class="home">
    <h2>个人主页</h2>
    <div v-if="user" class="card">
      <p><strong>账号 ID：</strong>{{ user.id }}</p>
      <p><strong>登录名：</strong>{{ user.name }}</p>
      <p><strong>角色：</strong>{{ (user.roles || []).join(', ') }}</p>
      <p><strong>权限：</strong>{{ (user.permissions || []).join(', ') }}</p>
    </div>
    <p v-else-if="error" class="error">{{ error }}</p>
    <p v-else>加载中...</p>
    <div class="actions">
      <router-link to="/user/list">查看用户列表（需要 user:list 权限）</router-link>
      <button @click="onLogout">退出登录</button>
    </div>
  </div>
</template>

<style scoped>
.card {
  padding: 1rem;
  border: 1px solid #eee;
  border-radius: 4px;
  margin: 1rem 0;
}
.error {
  color: #f56c6c;
}
.actions {
  display: flex;
  gap: 1rem;
  align-items: center;
  margin-top: 1.5rem;
}
button {
  padding: 0.5rem 1rem;
  background: #f56c6c;
  color: white;
  border: none;
  border-radius: 4px;
  cursor: pointer;
}
a {
  color: #409eff;
  text-decoration: none;
}
</style>
