# Sa-Token 文档适配说明（Java → Rust）

本文档说明 `sa-token-doc` 从 Java 官方文档迁移到 **Sa-Token-Rs** 后的约定。源目录一一对应拷贝自：

`Sa-Token/sa-token-doc` → `sa-token-rs/crates/sa-token-doc`

在保持 **文件路径 / 章节结构 / docsify 导航** 不变的前提下，内容按 Rust 实现改写。

---

## 1. 版本与仓库

| 项 | 值 |
|---|---|
| 文档包路径 | `crates/sa-token-doc` |
| 当前版本 | `0.1.0`（与 workspace 一致） |
| MSRV | Rust **1.88+**，edition **2024** |
| 权威实现说明 | 仓库根 `docs/GUIDE.md`、`docs/ARCHITECTURE.md`、`README.md` |
| 可运行示例 | `crates/sa-token-demo/*` |

本目录为 **Docsify 静态文档站**，不是可发布的业务 crate；`Cargo.toml` 仅作 workspace 成员占位。

---

## 2. 框架与技术栈映射

| Java / 原生态 | Sa-Token-Rs |
|---|---|
| Spring Boot / Spring MVC | **axum** + `sa-token-web-axum` |
| Spring WebFlux | **axum + tokio**（同适配层，异步 handler） |
| Solon / Quarkus | **actix-web** + `sa-token-web-actix` |
| （额外）Salvo | `sa-token-web-salvo` |
| Maven / Gradle | **Cargo**（`Cargo.toml`） |
| Jackson / Fastjson | **serde + serde_json** |
| FreeMarker 方言 | **Tera**（`sa-token-tera`） |
| Thymeleaf 方言 | **Askama**（`sa-token-askama`） |
| Redis + Jackson 序列化 | `sa-token-dao-redis` + serde |
| `application.yml` `sa-token.*` | `SaTokenConfig` + `SaManager::set_config` |
| Filter / Interceptor / Plugin | axum `SaTokenLayer` / actix `SaTokenMiddleware` |

文档内若仍出现「SpringBoot 示例」等标题，语义上已映射为 **axum**；「Solon」映射为 **actix-web**；「WebFlux」映射为 **axum 异步**。侧边栏 `_sidebar.md` 已按此更新。

---

## 3. API 命名对照（节选）

| Java | Rust |
|---|---|
| `StpUtil.login(id)` | `StpUtil::login(id)?` |
| `StpUtil.logout()` | `StpUtil::logout()?` |
| `StpUtil.isLogin()` | `StpUtil::is_login()` |
| `StpUtil.checkLogin()` | `StpUtil::check_login()?` |
| `StpUtil.getLoginId()` | `StpUtil::get_login_id()?` |
| `StpUtil.getTokenValue()` | `StpUtil::get_token_value()?` |
| `StpUtil.hasPermission("x")` | `StpUtil::has_permission("x")` |
| `StpUtil.checkPermission("x")` | `StpUtil::check_permission("x")?` |
| `@SaCheckLogin` | `#[sa_check_login]` |
| `@SaCheckRole("admin")` | `#[sa_check_role("admin")]` |
| `@SaCheckPermission("a")` | `#[sa_check_permission("a")]` |
| `@SaIgnore` | `#[sa_ignore]` |
| `SaTokenConfig` | `SaTokenConfig`（字段多为 snake_case） |
| `SaManager` | `SaManager` |

具体以 `sa-token` / `sa-token-core` 源码与 `docs/GUIDE.md` 为准。

---

## 4. 文档文件改写策略

1. **已优先重写**：`README.md`、`_sidebar.md`、`start/example.md`、`use/login-auth.md`、`use/session.md`、`use/at-check.md`、`use/config.md`、`use/kick.md`、`use/jur-auth.md`、`use/route-check.md`、`use/dao-extend.md`、`api/stp-util.md`、本文。
2. **批量已处理**：全站 `*.md` 中 `StpUtil.foo` → `StpUtil::snake_case`、` ```java ` → ` ```rust `、常见注解 → `#[sa_check_*]`。
3. **仍可能残留**：个别章节代码块仍含 Spring/`@RequestMapping` 等 Java 语法；以页首提示与 `sa-token-demo-*` 为准逐步精修。`more/update-log.md` 保留 Java 历史 changelog。
4. **不要删除** 1:1 拷贝的文件树（含 `static/`、`index.html`、`doc.html`），以免破坏 docsify 路径。

---

## 5. 本地预览

```bash
# 在 crates/sa-token-doc 下用任意静态服务器打开
cd crates/sa-token-doc
npx --yes serve -p 3000
# 浏览器访问 http://127.0.0.1:3000/
```

或使用 docsify-cli：

```bash
npx --yes docsify-cli serve crates/sa-token-doc -p 3000
```

---

## 6. 贡献约定

- 新增示例优先指向 `sa-token-demo-*`，避免文档内造无法编译的孤立片段。
- JSON 示例字段用 serde 可序列化结构；禁止再写 Jackson `ObjectMapper`。
- 注释 / 说明可中英双语，保持与仓库其它 crate 中文注释风格一致。
