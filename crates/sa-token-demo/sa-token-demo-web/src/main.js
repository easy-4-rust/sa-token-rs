// Vue3 entry: 1:1 镜像 Java 端 sa-token-demo-sso-client-vue3
import { createApp } from 'vue'
import { createPinia } from 'pinia'
import App from './App.vue'
import router from './router'

const app = createApp(App)
app.use(createPinia())
app.use(router)
app.mount('#app')
