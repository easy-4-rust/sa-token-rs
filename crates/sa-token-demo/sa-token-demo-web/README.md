# sa-token-demo-web

Vue3 前端 demo，1:1 镜像 Java Sa-Token 端 `sa-token-demo-sso-client-vue3` 的 UI/UX。

## 包含内容

- 登录页（`/login`）
- 个人主页（`/home`，需要登录）
- 用户列表（`/user/list`，需要 `user:list` 权限）
- 路由守卫（`router.beforeEach`）：从 `localStorage.satoken` 读取 token
- Axios 拦截器：自动附加 `satoken` header，处理 401/403

## 对应关系

| Vue3 demo 组件 | 对应 Java 端 |
|---|---|
| `views/LoginView.vue` | `LoginController.doLogin()` |
| `views/HomeView.vue` | `HomeController.userInfo()` |
| `views/UserView.vue` | `JurAuthController.userList()` |
| `api.js` | 客户端 sa-token 工具类 |
| `router.js` | `SaInterceptor` 路由拦截器 |

## 后端配合

```bash
# 1. 启动 sa-token-rs 后端（任选一个 axum/actix/salvo demo）
cd crates/sa-token-demo/sa-token-demo-axum
cargo run

# 2. 启动 Vue3 前端
cd crates/sa-token-demo/sa-token-demo-web
npm install
npm run dev
```

前端 dev server 默认在 `http://localhost:5173`，通过 Vite proxy 把 `/api/*` 转发到 `http://localhost:3000`（后端 axum demo 的端口）。

## 测试用账号

- `10001 / 123456`（普通用户）
- `10002 / 123456`（admin，拥有 `user:list` 权限）
