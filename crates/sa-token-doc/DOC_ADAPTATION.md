# Sa-Token-Rs 文档适配说明（Java → Rust）

本文说明如何将 Java 版 `sa-token-doc` 适配为 **Sa-Token-Rs** 文档。

源：`Sa-Token/sa-token-doc` → 目标：`sa-token-rs/crates/sa-token-doc`（文件路径一一对应）。

---

## 适配原则（必须遵守）

1. **不允许随意删减**：保留原文章节结构、说明文字、提示框、图片、练习链接；只替换技术内容。
2. **品牌替换**：叙述中的 `Sa-Token` → `Sa-Token-Rs`（Java 原站链接 https://sa-token.cc 可保留并标注「Java 原版」）。
3. **示例替换**：` ```java ` 示例改为等价 ` ```rust `；Maven/Gradle 改为 Cargo；Spring/WebFlux/Solon 映射为 axum / axum+tokio / actix-web。
4. **适当增加对照**：重要章节增加「Java ↔ Rust」对照表，必要时保留简短 Java 对照片段（标注为对照，不以 Java 为主示例）。
5. **API 以源码为准**：同步门面 `StpUtil::snake_case(...) -> SaResult`；异步门面见 `AsyncStpUtil`；Web 层 `sa-token-web-axum` / `sa-token-web-actix`。

---

## 技术栈映射

| Java | Sa-Token-Rs |
|---|---|
| Spring Boot / MVC | axum + `sa-token-web-axum` |
| Spring WebFlux | axum + tokio（或 `AsyncStpUtil`） |
| Solon / Quarkus | actix-web + `sa-token-web-actix` |
| Maven / Gradle | Cargo |
| Jackson / Fastjson | serde + serde_json |
| FreeMarker / Thymeleaf | Tera / Askama |
| `application.yml` | `SaTokenConfig` + `SaManager::set_config` |
| `SaInterceptor` / Filter | `SaTokenLayer` / `require_login` |
| `@SaCheckLogin` | `#[sa_check_login]` |

## API 映射（节选）

| Java | Rust |
|---|---|
| `StpUtil.login(10001)` | `StpUtil::login("10001")?` |
| `StpUtil.isLogin()` | `StpUtil::is_login()?` |
| `StpUtil.checkLogin()` | `StpUtil::check_login()?` |
| `StpUtil.getLoginId()` | `StpUtil::get_login_id()?` |
| `StpUtil.getTokenValue()` | `StpUtil::get_token_value()` |
| `StpUtil.logout()` | `StpUtil::logout()?` |
| `StpUtil.kickout(id)` | `StpUtil::kickout("id")?` |
| `StpUtil.checkPermission("x")` | `StpUtil::check_permission("x")?` |
| `@SaCheckPermission("x")` | `#[sa_check_permission("x")]` |

版本：workspace `0.1.0`，MSRV Rust 1.88+，edition 2024。

预览：`npx docsify-cli serve crates/sa-token-doc -p 3000`
