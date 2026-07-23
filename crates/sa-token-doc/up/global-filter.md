# 全局过滤器
--- 

> Sa-Token → Sa-Token-Rs。Spring `SaServletFilter` → axum `SaTokenLayer` + `SaRouter`；Solon / 其它 → actix 中间件同理。Jackson → `serde_json`。

| Java | Rust |
|---|---|
| `SaServletFilter` | `sa_token_web_axum::SaTokenLayer`（别名亦见 `SaServletFilter`） |
| `SaReactorFilter`（WebFlux） | 同样挂 `SaTokenLayer` / 异步 runtime |
| `setAuth` / `setError` / `setBeforeAuth` | 中间件闭包 + `tower_http` 响应头 / 统一错误处理 |
| `SaRouter.match` | `SaRouter::match_paths` |
| `FilterRegistrationBean.setOrder` | Tower Layer 注册顺序（后添加更靠外，按框架文档调整） |

### 组件简述

之前的章节中，我们学习了“根据拦截器实现路由拦截鉴权”，其实在大多数web框架中，使用过滤器可以实现同样的功能，本章我们就利用Sa-Token-Rs全局过滤器（中间件 Layer）来实现路由拦截器鉴权。

首先我们先梳理清楚一个问题，既然拦截器已经可以实现路由鉴权，为什么还要用过滤器再实现一遍呢？简而言之：
1. 相比于拦截器，过滤器更加底层，执行时机更靠前，有利于防渗透扫描。
2. 过滤器可以拦截静态资源，方便我们做一些权限控制。
3. 部分Web框架根本就没有提供拦截器功能，但几乎所有的Web框架都会提供过滤器 / 中间件机制。

但是过滤器也有一些缺点，比如：
1. 由于太过底层，导致无法率先拿到`HandlerMethod`对象，无法据此添加一些额外功能。
2. 由于拦截的太全面了，导致我们需要对很多特殊路由(如`/favicon.ico`)做一些额外处理。
3. 在 Spring 中，过滤器中抛出的异常无法进入全局`@ExceptionHandler`，我们必须额外编写代码进行异常处理；axum / actix 同样需要统一错误类型或中间件捕获。

Sa-Token-Rs同时提供过滤器和拦截器机制，不是为了让谁替代谁，而是为了让大家根据自己的实际业务合理选择，拥有更多的发挥空间。


### 在 axum 中注册过滤器（Layer）
同拦截器一样，为了避免不必要的性能浪费，你需要显式把 `SaTokenLayer` 挂到路由上（负责注入请求上下文）。鉴权逻辑可用 `SaRouter` 或单独的 auth Layer 完成：

``` rust
use axum::Router;
use axum::http::{HeaderName, HeaderValue};
use axum::routing::get;
use sa_token::prelude::*;
use sa_token_core::router::SaRouter;
use sa_token_web_axum::SaTokenLayer;
use tower_http::set_header::SetResponseHeaderLayer;

/// [Sa-Token-Rs 权限认证] 路由组装（对应 Java SaTokenConfigure + SaServletFilter）
fn build_router() -> Router {
    // 认证函数示意：登录认证 -- 拦截所有路由，并排除 /user/doLogin
    // 可放在中间件或各 handler 入口：
    let _ = SaRouter::match_paths(&["/**"])
        .not_match(&["/user/doLogin", "/favicon.ico"])
        .check(|_| StpUtil::check_login());

    Router::new()
        .route("/user/doLogin", get(|| async { "ok" }))
        // ---------- 设置一些安全响应头（对应 setBeforeAuth）----------
        .layer(SetResponseHeaderLayer::overriding(
            HeaderName::from_static("x-frame-options"),
            HeaderValue::from_static("SAMEORIGIN"),
        ))
        .layer(SetResponseHeaderLayer::overriding(
            HeaderName::from_static("x-xss-protection"),
            HeaderValue::from_static("1; mode=block"),
        ))
        .layer(SetResponseHeaderLayer::overriding(
            HeaderName::from_static("x-content-type-options"),
            HeaderValue::from_static("nosniff"),
        ))
        .layer(SetResponseHeaderLayer::overriding(
            HeaderName::from_static("server"),
            HeaderValue::from_static("sa-server"),
        ))
        // 注册 [Sa-Token-Rs 全局过滤器 / 上下文 Layer]
        .layer(SaTokenLayer::new())
}
```

> [!WARNING| label:注意事项：] 
> - 在`[认证函数]`里，你可以写和拦截器里一致的代码，进行路由匹配鉴权，参考：[路由拦截鉴权](/use/route-check)。
> - 由于中间件中抛出的异常不一定进入统一错误处理，所以你必须提供`[异常处理函数]` / `IntoResponse` 来处理`[认证函数]`里抛出的异常。
> - 在`[异常处理函数]`里的返回值，将作为 JSON / 字符串输出到前端，如果需要定制化返回数据，请注意其中的格式转换（`serde_json`）。

改写错误响应格式示例：
``` rust
use axum::Json;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use sa_token_core::exception::SaTokenException;
use serde_json::json;

/// 将认证异常转为 JSON（对应 Java setError + JSONUtil）
fn auth_error(e: SaTokenException) -> impl IntoResponse {
    (
        StatusCode::UNAUTHORIZED,
        [(
            axum::http::header::CONTENT_TYPE,
            "application/json;charset=UTF-8",
        )],
        Json(json!({ "code": 500, "msg": e.to_string() })),
    )
}
```

JSON 工具对应 Java Hutool-Json → Rust `serde` / `serde_json`。


### 自定义过滤器执行顺序

Java 中 `SaServletFilter` 默认 Order 为 `-100`，可用 `FilterRegistrationBean` 调整。

在 axum / Tower 中，**Layer 的添加顺序**决定执行时机（后 `.layer(...)` 的更靠外、更先执行）。若要让安全响应头或鉴权早于业务逻辑，按框架约定调整 `.layer` 顺序即可：

``` rust
use axum::Router;
use sa_token_web_axum::SaTokenLayer;

fn ordered_router() -> Router {
    Router::new()
        // ... routes ...
        .layer(SaTokenLayer::new()) // 按需调整相对其它 Layer 的位置
}
```

在 SpringBoot 中， Order 值越小，执行时机越靠前；在 Tower 中请以官方 Layer 顺序规则为准。


### 在异步 Web / 其它框架中注册过滤器
`Spring WebFlux` 中不提供拦截器机制，因此若你的项目需要路由鉴权功能，过滤器是你唯一的选择。Rust 侧无论 axum 还是 actix-web，都通过中间件完成同等能力：
- **axum**：`SaTokenLayer`（见上）。
- **actix-web**（对应 Java Solon 等轻量栈的映射习惯）：使用 `sa-token-web-actix` 提供的中间件 / 等价 Layer，并在认证闭包中调用 `StpUtil::check_login()` 或 `SaRouter`。

``` rust
// actix-web 示意（具体类型以 sa-token-web-actix 导出为准）
// App::new()
//   .wrap(SaTokenMiddleware::new())
//   .configure(|cfg| { /* routes */ })
```
		
---

<a class="case-btn" href="https://github.com/sa-token-rust/sa-token-rs/tree/main/crates/sa-token-demo/sa-token-demo-axum"
	target="_blank">
	本章代码示例：Sa-Token-Rs 全局过滤器 —— [ SaTokenLayer + demo-axum ]
</a>
