import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'

// 1:1 镜像 Java 端 sa-token-demo-sso-client-vue3 的 Vite 配置
export default defineConfig({
  plugins: [vue()],
  server: {
    port: 5173,
    proxy: {
      // 代理到 sa-token-rs 后端（axum / actix / salvo 任一）
      '/api': {
        target: 'http://localhost:3000',
        changeOrigin: true,
      },
    },
  },
})
