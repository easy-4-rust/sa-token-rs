# actix-web 集成 Sa-Token-Rs 示例

> Java 原文对应：`Solon 集成 Sa-Token 示例`  
> 框架映射：**Solon / Quarkus → actix-web**（`sa-token-web-actix`）

本篇介绍在 actix-web 应用中如何集成 Sa-Token-Rs。

整合示例在官方仓库的 `crates/sa-token-demo/sa-token-demo-actix-web` 文件夹下，如遇到难点可结合源码进行学习测试。[Sa-Token-Rs 集成示例大全](/more/download-demos) 。

| Java | Rust |
|---|---|
| Solon | actix-web |
| `sa-token-solon-plugin` | `sa-token-web-actix` |
| `@SolonMain` | `#[actix_web::main]` |
| `@Mapping` / `@Controller` | `web::scope` + handler |

> [!tip| label:为什么是 actix-web？]
> Java 文档中 Solon 定位为轻量、高性能 Web 框架。Rust 侧与之对应的常用选择是 **actix-web**（另一主流为 axum，见 [axum 集成](/start/example)）。
>
> - 成熟的 actor / 异步模型；
> - 丰富的中间件生态；
> - 与 `AsyncStpUtil` 配合良好。
>
> Java Solon 官网仍可参考：[https://solon.noear.org/](https://solon.noear.org/)

---

### 1、创建项目

使用 Cargo 新建一个 actix-web 项目，例如：`sa-token-demo-actix-web`

```bash
cargo new sa-token-demo-actix-web --bin
```

### 2、添加依赖

在项目中添加依赖：

<!---------------------------- tabs:start ---------------------------->
<!-------- tab:Cargo.toml -------->
``` toml
[dependencies]
sa-token = "0.1"
sa-token-web-actix = "0.1"
actix-web = "4"
tokio = { version = "1", features = ["full"] }
serde_json = "1"
```

<!-------- tab:Java 对照（Maven） -------->
``` xml
<!-- 以下为 Java 原版，仅作对照 -->
<dependency>
    <groupId>cn.dev33</groupId>
    <artifactId>sa-token-solon-plugin</artifactId>
    <version>${sa.top.version}</version>
</dependency>
```
<!---------------------------- tabs:end ---------------------------->



Cargo 依赖一直无法加载成功？[参考解决方案](/start/maven-pull)

更多内测版本了解：[Sa-Token-Rs 最新版本](/start/new-version.md)



### 3、设置配置

你可以**零配置启动项目**，但同时你也可以在代码中定制 `SaTokenConfig`（对应 Java `app.yml` 中的 `sa-token.*`）：

<!---------------------------- tabs:start ---------------------------->

<!------------- tab:Rust 代码配置  ------------->

``` rust
use std::sync::Arc;
use sa_token::prelude::{AsyncSaTokenRuntime, AsyncStpUtil, SaTokenConfig, SaTokenDaoMemory};
use sa_token_core::context::sa_token_context_default_impl::SaTokenContextDefaultImpl;

/// 构建 AsyncStpUtil（actix demo 常用异步门面）
fn build_stp_util() -> AsyncStpUtil {
    let runtime = AsyncSaTokenRuntime::new(
        Arc::new(SaTokenConfig {
            token_name: "satoken".into(),
            timeout: 2_592_000,
            active_timeout: -1,
            is_concurrent: true,
            is_share: false,
            is_log: true,
            ..Default::default()
        }),
        Arc::new(SaTokenDaoMemory::new()),
        Arc::new(SaTokenContextDefaultImpl),
    );
    AsyncStpUtil::new("login", Arc::new(runtime))
}
```

<!------------- tab:YAML 语义对照（Java app.yml）  ------------->

```yaml
server:
    port: 8081

############## Sa-Token 配置语义对照 ##############
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

``` rust
use actix_web::{web, App, HttpServer};
use sa_token_web_actix::require_login;
use actix_web::middleware::from_fn;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let util = build_stp_util();
    println!("启动成功，Sa-Token-Rs actix 示例");

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(util.clone()))
            .route("/user/doLogin", web::get().to(do_login))
            .route("/user/isLogin", web::get().to(is_login))
            // 需登录接口可包一层 require_login
            // .service(web::scope("/user").wrap(from_fn(require_login)).route(...))
    })
    .bind(("0.0.0.0", 8081))?
    .run()
    .await
}
```

### 5、创建测试 Handler（对应 Controller）

``` rust
use actix_web::{web, HttpResponse};
use sa_token::prelude::AsyncStpUtil;
use std::collections::HashMap;

// 测试登录，浏览器访问： http://localhost:8081/user/doLogin?username=zhang&password=123456
async fn do_login(
    util: web::Data<AsyncStpUtil>,
    query: web::Query<HashMap<String, String>>,
) -> HttpResponse {
    let username = query.get("username").map(|s| s.as_str()).unwrap_or("");
    let password = query.get("password").map(|s| s.as_str()).unwrap_or("");
    // 此处仅作模拟示例，真实项目需要从数据库中查询数据进行比对
    if username == "zhang" && password == "123456" {
        match util.login("10001").await {
            Ok(_) => HttpResponse::Ok().body("登录成功"),
            Err(e) => HttpResponse::Unauthorized().body(e.to_string()),
        }
    } else {
        HttpResponse::Ok().body("登录失败")
    }
}

// 查询登录状态，浏览器访问： http://localhost:8081/user/isLogin
async fn is_login(util: web::Data<AsyncStpUtil>) -> HttpResponse {
    let logged_in = util.is_login().await.unwrap_or(false);
    HttpResponse::Ok().body(format!("当前会话是否登录：{logged_in}"))
}
```

### 6、运行

```bash
cargo run -p sa-token-demo-actix-web
```

从浏览器依次访问上述测试接口：

<img src="/big-file/doc/start/test-do-login.png" alt="运行结果">


<img src="/big-file/doc/start/test-is-login.png" alt="运行结果">


### 详细了解

通过这个示例，你已经对 Sa-Token-Rs 有了初步的了解，那么现在开始详细了解一下它都有哪些吧：

[登录认证](/use/login-auth) (与 axum 处理类似)
