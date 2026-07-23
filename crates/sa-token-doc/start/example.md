# axum 集成 Sa-Token-Rs 示例

> Java 原文对应：`SpringBoot 集成 Sa-Token 示例`  
> 框架映射：**Spring Boot → axum**（`sa-token-web-axum`）

| Java | Rust (Sa-Token-Rs) |
|---|---|
| Spring Boot | axum + tokio |
| `sa-token-spring-boot3-starter` | `sa-token` + `sa-token-web-axum` |
| `application.yml` | `SaTokenConfig` + `SaManager::set_config` |
| `@RestController` | axum handler + `Router` |

本篇带你从零开始集成 Sa-Token-Rs，只需简单 5 步，你就可以快速熟悉框架的使用姿势。

整合示例在官方仓库的 `crates/sa-token-demo/sa-token-demo-axum` 文件夹下，如遇到难点可结合源码进行学习测试。[Sa-Token-Rs 集成示例大全](/more/download-demos) 。

---

### 1、创建项目

使用 Cargo 新建一个 Rust 项目，例如：`sa-token-demo-axum`

```bash
cargo new sa-token-demo-axum --bin
cd sa-token-demo-axum
```

（不会的同学可参考 [The Cargo Book](https://doc.rust-lang.org/cargo/) 或仓库内已有 demo）


### 2、添加依赖

在项目中添加依赖：

<!---------------------------- tabs:start ---------------------------->
<!-------- tab:Cargo.toml（推荐） -------->
``` toml
[dependencies]
# Sa-Token-Rs 权限认证（对应 Java sa-token-spring-boot-starter）
sa-token = { version = "0.1", path = "../../sa-token" }  # monorepo 内用 path；发布后可写 version
sa-token-web-axum = { version = "0.1", path = "../../sa-token-web/sa-token-web-axum" }
axum = "0.8"
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
```

- 若使用 **actix-web**（对应 Java Solon / Quarkus），请引入 `sa-token-web-actix`，见 [actix 集成](/start/solon-example)。
- 若使用 **Salvo**，请引入 `sa-token-web-salvo`。

<!-------- tab:Java 对照（Maven） -------->
``` xml
<!-- 以下为 Java 原版依赖，仅作对照，Rust 项目请用上方 Cargo.toml -->
<dependency>
	<groupId>cn.dev33</groupId>
	<artifactId>sa-token-spring-boot3-starter</artifactId>
	<version>${sa.top.version}</version>
</dependency>
```
<!---------------------------- tabs:end ---------------------------->


Cargo 依赖一直无法加载成功？[参考解决方案](/start/maven-pull)

更多版本了解：[Sa-Token-Rs 最新版本](/start/new-version.md)

### 3、设置配置

你可以**零配置启动项目**，但同时你也可以在启动代码中定制 `SaTokenConfig`（对应 Java `application.yml` 中的 `sa-token.*`）：

<!---------------------------- tabs:start ---------------------------->

<!------------- tab:Rust 代码配置（推荐）  ------------->
``` rust
use std::sync::Arc;
use sa_token::prelude::*;

/// 初始化配置（对应 Java application.yml 中的 sa-token 段）
fn init_sa_token() {
    SaManager::set_config(Arc::new(SaTokenConfig {
        // token 名称（同时也是 cookie / header 名称）
        token_name: "satoken".into(),
        // token 有效期（单位：秒） 默认30天，-1 代表永久有效
        timeout: 2_592_000,
        // token 最低活跃频率（单位：秒），-1 代表不限制
        active_timeout: -1,
        // 是否允许同一账号多地同时登录
        is_concurrent: true,
        // 多人登录同一账号时是否共用一个 token
        is_share: false,
        // token 风格：uuid / simple-uuid / random-32 / ...
        token_style: SaTokenStyle::Uuid,
        // 是否输出操作日志
        is_log: true,
        ..Default::default()
    }));
    SaManager::set_sa_token_dao(Arc::new(SaTokenDaoMemory::new()));
    SaManager::put_stp_logic(Arc::new(StpLogic::new("login")));
}
```

<!------------- tab:YAML 语义对照（Java 风格）  ------------->
``` yaml
# 以下为 Java application.yml 语义对照，Rust 中请用上方 SaTokenConfig 字段
server:
    port: 8081

sa-token:
	token-name: satoken
	timeout: 2592000
	active-timeout: -1
	is-concurrent: true
	is-share: false
	token-style: uuid
	is-log: true
```

<!---------------------------- tabs:end ---------------------------->


### 4、创建启动入口

在 `src/main.rs` 中写入：

``` rust
use std::sync::Arc;
use axum::{routing::get, Router};
use sa_token::prelude::*;
use sa_token_web_axum::SaTokenLayer;

#[tokio::main]
async fn main() {
    init_sa_token(); // 见上一节

    let app = Router::new()
        .route("/user/doLogin", get(do_login))
        .route("/user/isLogin", get(is_login))
        .layer(SaTokenLayer::new());

    println!("启动成功，Sa-Token-Rs 已初始化");
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8081").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
```

| Java | Rust |
|---|---|
| `@SpringBootApplication` + `main` | `#[tokio::main]` + `axum::serve` |
| `SaManager.getConfig()` | `SaManager` 已通过 `set_config` 注入 |


### 5、创建测试 Handler（对应 Controller）

``` rust
use axum::extract::Query;
use sa_token::prelude::*;
use serde::Deserialize;
use std::collections::HashMap;

// 测试登录，浏览器访问： http://localhost:8081/user/doLogin?username=zhang&password=123456
async fn do_login(Query(params): Query<HashMap<String, String>>) -> String {
    let username = params.get("username").map(|s| s.as_str()).unwrap_or("");
    let password = params.get("password").map(|s| s.as_str()).unwrap_or("");
    // 此处仅作模拟示例，真实项目需要从数据库中查询数据进行比对
    if username == "zhang" && password == "123456" {
        let _ = StpUtil::login("10001");
        "登录成功".into()
    } else {
        "登录失败".into()
    }
}

// 查询登录状态，浏览器访问： http://localhost:8081/user/isLogin
async fn is_login() -> String {
    format!(
        "当前会话是否登录：{}",
        StpUtil::is_login().unwrap_or(false)
    )
}
```

| Java | Rust |
|---|---|
| `StpUtil.login(10001)` | `StpUtil::login("10001")?` |
| `StpUtil.isLogin()` | `StpUtil::is_login()?` |
| `@RequestMapping("doLogin")` | `.route("/user/doLogin", get(do_login))` |

### 6、运行

```bash
cargo run -p sa-token-demo-axum
# 或在本示例目录：cargo run
```

从浏览器依次访问上述测试接口：

<img src="/big-file/doc/start/test-do-login.png" alt="运行结果">

<img src="/big-file/doc/start/test-is-login.png" alt="运行结果">

<!--
### 无 Web 框架环境
若仅使用 core（无 axum/actix），需自行实现 / 注入 SaTokenContext，参考：[自定义 SaTokenContext 指南](/fun/sa-token-context)
-->


### 出发

通过这个示例，你已经对 Sa-Token-Rs 有了初步的了解。那么，坐稳扶好，让我们开始吧：[登录认证](/use/login-auth)
