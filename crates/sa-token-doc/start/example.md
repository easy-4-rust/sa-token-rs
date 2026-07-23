# Axum 集成 Sa-Token-Rs 示例

> ⚠️ **本文档适配状态**：✅ 已完成 Rust 移植版
>
> 本文档基于 Java 版的 `start/example.md` 移植，Java 源码保留在 [`Sa-Token/sa-token-doc/start/example.md`](https://gitee.com/dromara/sa-token/blob/dev/sa-token-doc/start/example.md)。
>
> | Java (sa-token) | Rust (sa-token-rs) |
> | --- | --- |
> | SpringBoot 3.x / 4.x | **Axum 0.7+** |
> | `cn.dev33:sa-token-spring-boot3-starter` | `sa-token-rs = { version = "0.1", features = ["axum"] }` |
> | `@RestController` | `axum::Router` + handler 函数 |
> | `application.yml` | `SaTokenConfig::builder()` |

本篇带你从零开始集成 Sa-Token-Rs，只需简单 5 步，你就可以快速熟悉框架的使用姿势。

整合示例在官方仓库的`/sa-token-demo/sa-token-demo-axum`文件夹下，如遇到难点可结合源码进行学习测试。

---

### 1、创建项目

使用 `cargo new` 创建一个新的 Rust 项目：

```bash
cargo new sa-token-demo-axum
cd sa-token-demo-axum
cargo add sa-token --features axum
cargo add axum tokio --features tokio/full
```

> 如果 `cargo add` 网络不通，可手动编辑 `Cargo.toml`（详见 [download.md](./download.md)）。

### 2、添加依赖

在 `Cargo.toml` 中添加依赖：

```toml
[package]
name = "sa-token-demo-axum"
version = "0.1.0"
edition = "2024"

[dependencies]
sa-token = { version = "0.1", features = ["axum"] }
axum = "0.7"
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
```

> - 如果你使用其它 Web 框架（Actix-web / Salvo / Rocket），引入对应的 starter：`sa-token-actix`、`sa-token-salvo` 等。

依赖一直无法加载成功？参考 [Cargo 依赖问题解决方案](./maven-pull.md)（标题保留兼容，内部内容已改为 Cargo）。

### 3、设置配置

你可以**零配置启动项目** ，但同时你也可以在代码中以 `SaTokenConfig::builder()` 方式定制：

```rust
use std::sync::Arc;
use sa_token::SaTokenConfig;
use sa_token::sa_manager::SaManager;
use sa_token::stp::StpLogic::StpLogic;

#[tokio::main]
async fn main() {
    // 1. 初始化全局配置
    let config = SaTokenConfig::builder()
        .token_name("satoken")                    // token 名称（同时也是 cookie 名称）
        .timeout(30 * 24 * 60 * 60)               // token 有效期（秒），默认30天，-1 代表永久有效
        .active_timeout(-1)                        // token 最低活跃频率（秒），-1 代表不限制
        .is_concurrent(true)                       // 是否允许同一账号多地同时登录
        .is_share(false)                           // 多人登录同一账号时是否共用一个 token
        .token_style("uuid")                       // uuid / simple-uuid / random-32 / random-64 / random-128 / tik
        .is_log(true)                              // 是否输出操作日志
        .build();

    SaManager::set_config(Arc::new(config));

    // 2. 注册 StpLogic（"login" 是账号类型，可自定义如 "admin"、"user" 等）
    SaManager::put_stp_logic(Arc::new(StpLogic::new("login")));

    // 3. 启动 Axum 服务
    let app = axum::Router::new()
        .route("/user/doLogin", axum::routing::get(user_do_login))
        .route("/user/isLogin", axum::routing::get(user_is_login));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8081").await.unwrap();
    println!("启动成功，Sa-Token 配置：{}", SaManager::config());
    axum::serve(listener, app).await.unwrap();
}
```

> **等价的 application.yml 配置**（如果你更熟悉 YAML 风格）：
>
> ```yaml
> ############## Sa-Token 配置 ##############
> sa-token:
>   token-name: satoken
>   timeout: 2592000         # 30 天
>   active-timeout: -1       # 不限制
>   is-concurrent: true
>   is-share: false
>   token-style: uuid
>   is-log: true
> ```
>
> 在 Rust 中你需要转换为 `SaTokenConfig::builder()` 链式调用。

### 4、创建业务 Handler

```rust
use axum::Json;
use sa_token::stp::StpUtil;
use serde_json::{json, Value};

// 测试登录，浏览器访问： http://localhost:8081/user/doLogin?username=zhang&password=123456
async fn user_do_login(
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> Json<Value> {
    let username = params.get("username").map(|s| s.as_str()).unwrap_or("");
    let password = params.get("password").map(|s| s.as_str()).unwrap_or("");

    // 此处仅作模拟示例，真实项目需要从数据库中查询数据进行比对
    if username == "zhang" && password == "123456" {
        StpUtil::login("10001").unwrap();
        Json(json!({ "msg": "登录成功" }))
    } else {
        Json(json!({ "msg": "登录失败" }))
    }
}

// 查询登录状态，浏览器访问： http://localhost:8081/user/isLogin
async fn user_is_login() -> Json<Value> {
    Json(json!({
        "isLogin": StpUtil::is_login()
    }))
}
```

### 5、运行

```bash
cargo run
```

从浏览器依次访问：

- `http://localhost:8081/user/doLogin?username=zhang&password=123456` → 返回 `{"msg":"登录成功"}`
- `http://localhost:8081/user/isLogin` → 返回 `{"isLogin":true}`

### 出发

通过这个示例，你已经对 Sa-Token-Rs 有了初步的了解。那么，坐稳扶好，让我们开始吧：[登录认证](/use/login-auth)

---

## 与 Java 版的关键差异

| 方面 | Java (sa-token) | Rust (sa-token-rs) |
| --- | --- | --- |
| Web 框架 | SpringBoot | Axum（另有 Actix/Salvo/Rocket starter） |
| 依赖管理 | Maven / Gradle | Cargo |
| 启动方式 | `@SpringBootApplication` | `#[tokio::main]` + `axum::serve` |
| 配置 | `application.yml` | `SaTokenConfig::builder()` Rust DSL |
| 登录注解 | `@SaCheckLogin` | `#[sa_check_login]`（proc-macro） |
| 自动注入 | Spring Bean | 手动 `SaManager::set_*` |
| Cookie | `HttpServletResponse.addCookie` | `Set-Cookie` header（在 SaTokenContext 中实现） |

> 注：本示例使用最简同步风格的 axum handler，生产项目可按需使用 `axum::extract::State`、`axum::middleware` 等异步模式。详细见 [sa-token-axum 文档](../../sa-token-axum/)。