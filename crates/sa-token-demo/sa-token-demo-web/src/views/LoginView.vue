<script setup>
import { ref } from 'vue'
import { useRouter, useRoute } from 'vue-router'
import { login } from '../api.js'

const id = ref('10001')
const password = ref('123456')
const error = ref('')
const router = useRouter()
const route = useRoute()

async function onSubmit() {
  error.value = ''
  try {
    const resp = await login(id.value, password.value)
    // 后端返回 { token: "xxx" }
    if (resp.token) {
      localStorage.setItem('satoken', resp.token)
      const redirect = route.query.redirect || '/home'
      router.push(redirect)
    } else {
      error.value = '登录失败：未返回 token'
    }
  } catch (err) {
    error.value = err.response?.data?.message || err.message || '登录失败'
  }
}
</script>

<template>
  <div class="login">
    <h2>登录</h2>
    <form @submit.prevent="onSubmit">
      <label>
        账号:
        <input v-model="id" placeholder="10001" />
      </label>
      <label>
        密码:
        <input type="password" v-model="password" />
      </label>
      <button type="submit">登录</button>
      <p v-if="error" class="error">{{ error }}</p>
    </form>
    <p class="hint">默认账号: 10001 / 123456</p>
  </div>
</template>

<style scoped>
.login {
  max-width: 400px;
  margin: 2rem auto;
  padding: 2rem;
  border: 1px solid #ddd;
  border-radius: 8px;
}
form {
  display: flex;
  flex-direction: column;
  gap: 1rem;
}
label {
  display: flex;
  flex-direction: column;
  gap: 0.25rem;
}
input {
  padding: 0.5rem;
  font-size: 1rem;
  border: 1px solid #ccc;
  border-radius: 4px;
}
button {
  padding: 0.5rem 1rem;
  background: #409eff;
  color: white;
  border: none;
  border-radius: 4px;
  cursor: pointer;
  font-size: 1rem;
}
.error {
  color: #f56c6c;
}
.hint {
  color: #999;
  font-size: 0.85rem;
}
</style>
