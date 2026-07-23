<script setup>
import { RouterView } from 'vue-router'
</script>

<template>
  <div class="app">
    <header>
      <h1>sa-token-rs Vue3 demo</h1>
      <p>登录认证 + 权限校验（对应 Java sa-token 的 Vue3 演示）</p>
    </header>
    <main>
      <RouterView />
    </main>
  </div>
</template>

<style scoped>
.app {
  font-family: 'Helvetica Neue', Helvetica, Arial, sans-serif;
  max-width: 960px;
  margin: 0 auto;
  padding: 1.5rem;
}
header h1 {
  margin: 0 0 0.5rem 0;
}
</style>
