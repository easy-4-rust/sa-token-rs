# axum 异步场景集成 Sa-Token-Rs 示例

> Java 原文对应：`Spring WebFlux 集成 Sa-Token 示例`  
> 框架映射：**WebFlux / Reactor → axum + tokio**

**tokio** 是 Rust 主流异步运行时，本篇以 **axum** 为例，展示 Sa-Token-Rs 在异步 Web 中的整合方式。  
**你可以用同样思路对接其它异步框架（例如 tower 中间件栈、网关层等）。**

整合示例在官方仓库的 `crates/sa-token-demo/sa-token-demo-axum-async` 文件夹下，如遇到难点可结合源码进行测试学习。[Sa-Token-Rs 集成示例大全](/more/download-demos) 。

| Java | Rust |
|---|---|
| WebFlux + Reactor | axum + tokio |
| `sa-token-reactor-spring-boot-starter` | `sa-token-web-axum` |
| `SaReactorFilter` | `SaTokenLayer` |
| `Mono` / `Flux` | `async fn` + `SaResult` |

> [!WARNING| label:小提示 ]
> WebFlux 常用于微服务网关架构中，如果您的应用基于单体架构且非异步模型，可以先跳过本章；日常 axum 同步门面 `StpUtil` 也可在 `async fn` 中直接调用。


---

### 1、创建项目

使用 Cargo 新建一个项目，例如：`sa-token-demo-axum-async`

```bash
cargo new sa-token-demo-axum-async --bin
```


### 2、添加依赖

在项目中添加依赖：

<!---------------------------- tabs:start ------------------------------>
<!-------- tab:Cargo.toml -------->
``` toml
[dependencies]
# Sa-Token-Rs 权限认证（异步 Web 集成），对应 Java reactor starter
sa-token = "0.1"
sa-token-web-axum = "0.1"
axum = "0.8"
tokio = { version = "1", features = ["full"] }
```

<!-------- tab:Java 对照（Maven） -------->
``` xml
<!-- 以下为 Java 原版，仅作对照 -->
<dependency>
	<groupId>cn.dev33</groupId>
	<artifactId>sa-token-reactor-spring-boot-starter</artifactId>
	<version>${sa.top.version}</version>
</dependency>
```
<!---------------------------- tabs:end ------------------------------>





### 3、创建启动入口

在 `src/main.rs` 写入：

``` rust
use std::sync::Arc;
use axum::{routing::get, Router};
use sa_token::prelude::*;
use sa_token_web_axum::SaTokenLayer;

#[tokio::main]
async fn main() {
    SaManager::set_config(Arc::new(SaTokenConfig::default()));
    SaManager::set_sa_token_dao(Arc::new(SaTokenDaoMemory::new()));
    SaManager::put_stp_logic(Arc::new(StpLogic::new("login")));

    let app = Router::new()
        .route("/user/doLogin", get(do_login))
        .route("/user/isLogin", get(is_login))
        .layer(SaTokenLayer::new());

    println!("启动成功，Sa-Token-Rs 已初始化");
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8081").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
```


### 4、创建全局 Layer（对应全局过滤器）

新建配置模块，注册 Sa-Token-Rs 的全局 Layer（对应 Java `SaReactorFilter`）：

``` rust
use axum::Router;
use sa_token_web_axum::SaTokenLayer;

/// 注册 [Sa-Token-Rs 全局 Layer]
/// 对应 Java SaReactorFilter：拦截路由 / 放行路由 / 认证函数 / 异常处理
fn with_sa_token_layer(router: Router) -> Router {
    router.layer(SaTokenLayer::new())
    // 更细的路由放行：把公开接口放在未包裹 require 的分支上
    // 更细的鉴权：在 handler 内 StpUtil::check_login()? 或使用注解宏
}
```

| Java `SaReactorFilter` | Rust |
|---|---|
| `addInclude("/**")` | 整棵 Router 加 `SaTokenLayer` |
| `addExclude("/favicon.ico")` | 静态资源路由不加鉴权中间件 |
| `setAuth(...)` | handler / middleware 内 `check_*` |
| `setError(...)` | axum `IntoResponse` / 错误映射 |

你只需要按照此格式复制代码即可，有关过滤器的详细用法，会在之后的章节详细介绍。


### 5、创建测试 Handler（对应 Controller）

``` rust
use axum::extract::Query;
use sa_token::prelude::*;
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

### 6、运行

```bash
cargo run -p sa-token-demo-axum-async
```

从浏览器依次访问上述测试接口：

<img src="/big-file/doc/start/test-do-login.png" alt="运行结果">

<img src="/big-file/doc/start/test-is-login.png" alt="运行结果">


**注意事项：**

更多使用示例请参考官方仓库 demo（含 `AsyncStpUtil` 的完整异步门面用法）。
