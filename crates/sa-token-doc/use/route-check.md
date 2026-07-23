# 路由拦截鉴权

> 保留 Java 原文章节结构。`SaInterceptor` / `SaRouter` 在 Sa-Token-Rs 中对应 **axum `SaTokenLayer` + 路由分组 / middleware**，或 actix `require_login`。

| Java | Rust |
|---|---|
| `SaInterceptor` | `SaTokenLayer`（axum）/ `from_fn(require_login)`（actix） |
| `SaRouter.match(...).check(...)` | `Router::nest` + handler 内 `check_*`，或自定义 middleware |
| `excludePathPatterns` | 公开路由不挂鉴权中间件 / `#[sa_ignore]` |
| `@SaIgnore` | `#[sa_ignore]` |

假设我们有如下需求：<u>*项目中所有接口均需要登录校验，只有 “登录接口” 本身对外开放*</u>。

如果给每个接口都手动加上注解鉴权，将会是一件比较麻烦的事情，这时候使用拦截器鉴权模式将大大降低我们的代码量。

<!-- ![基础-拦截器鉴权.svg](../big-file/use/use-route-check.svg 'w-100') -->

<img class="w-100" src="/big-file/doc/use/use-route-check.svg" />

如上图所示，拦截器将拦截除登录以外的所以请求，并进行一道前置审核决定是否通过。

--- 


### 1、注册 Sa-Token-Rs 路由拦截器

以 **axum** 为例（对应 Java SpringBoot + `WebMvcConfigurer`）：

``` rust
use axum::{routing::{get, post}, Router};
use sa_token::prelude::*;
use sa_token_web_axum::SaTokenLayer;

/// 注册拦截：除登录外均需 check_login（对应 SaInterceptor + excludePathPatterns）
fn build_router() -> Router {
    Router::new()
        .route("/user/doLogin", post(do_login)) // 公开：不强制登录
        .merge(
            Router::new()
                .route("/user/info", get(user_info))
                // 其它需登录路由...
                .layer(axum::middleware::from_fn(require_login_mw)),
        )
        .layer(SaTokenLayer::new())
}

async fn require_login_mw(
    req: axum::extract::Request,
    next: axum::middleware::Next,
) -> Result<axum::response::Response, axum::response::Response> {
    StpUtil::check_login().map_err(|e| {
        (
            axum::http::StatusCode::UNAUTHORIZED,
            e.to_string(),
        )
            .into_response()
    })?;
    Ok(next.run(req).await)
}

async fn do_login() -> &'static str { "login" }
async fn user_info() -> SaResult<String> {
    StpUtil::check_login()?;
    StpUtil::get_login_id()
}
```

以上代码注册了基于 `StpUtil::check_login()` 的登录校验，并且把 `/user/doLogin` 放在未包裹登录中间件的公开路由上（除登录以外的接口都需要登录才能访问）。

actix-web 写法见 [actix 集成](/start/solon-example)：对需登录的 `web::scope` 使用 `.wrap(from_fn(require_login))`。

> [!WARNING| label:版本升级]
> Java 侧 `SaInterceptor` 是较新拦截器；Rust 侧请统一使用当前仓库的 `SaTokenLayer` / `require_login`，不要混用过时命名。


### 2、校验函数详解

自定义认证规则：只做登录校验是最简单的写法。

我们也可以按模块定义更详细的校验规则，例如（对应 Java `SaRouter.match` 链）：

``` rust
use axum::{routing::get, Router};
use sa_token::prelude::*;

async fn user_mod() -> SaResult<&'static str> {
    StpUtil::check_permission("user")?;
    Ok("user")
}
async fn admin_mod() -> SaResult<&'static str> {
    StpUtil::check_permission("admin")?;
    Ok("admin")
}
async fn goods_mod() -> SaResult<&'static str> {
    StpUtil::check_permission("goods")?;
    Ok("goods")
}
async fn orders_mod() -> SaResult<&'static str> {
    StpUtil::check_permission("orders")?;
    Ok("orders")
}
async fn notice_mod() -> SaResult<&'static str> {
    StpUtil::check_permission("notice")?;
    Ok("notice")
}
async fn comment_mod() -> SaResult<&'static str> {
    StpUtil::check_permission("comment")?;
    Ok("comment")
}

fn build_modular_router() -> Router {
    Router::new()
        .route("/user/doLogin", get(|| async { "login" }))
        .nest("/user", Router::new().route("/list", get(user_mod)))
        .nest("/admin", Router::new().route("/panel", get(admin_mod)))
        .nest("/goods", Router::new().route("/list", get(goods_mod)))
        .nest("/orders", Router::new().route("/list", get(orders_mod)))
        .nest("/notice", Router::new().route("/list", get(notice_mod)))
        .nest("/comment", Router::new().route("/list", get(comment_mod)))
}
```

Java `SaRouter.match()` 匹配函数有两个参数：
- 参数一：要匹配的 path 路由。
- 参数二：要执行的校验函数。

在校验函数内不只可以使用 `StpUtil::check_permission("xxx")`，你还可以写任意代码，例如：

``` rust
async fn admin_panel() -> SaResult<&'static str> {
    // 角色校验 -- 必须具备 admin 或 super-admin
    StpUtil::check_role_or(&["admin", "super-admin"])?;
    Ok("admin ok")
}

async fn any_logged() -> SaResult<&'static str> {
    StpUtil::check_login()?;
    println!("----啦啦啦----");
    Ok("ok")
}
```

Java 原文连缀示例对照：

``` java
SaRouter.match("/**", "/user/doLogin", r -> StpUtil.checkLogin());
SaRouter.match("/admin/**", r -> StpUtil.checkRoleOr("admin", "super-admin"));
SaRouter.match("/user/**", r -> StpUtil.checkPermission("user"));
```


### 3、匹配特征详解

除了 path 路由匹配，Java `SaRouter` 还支持很多其它特征。Rust 侧用 **路由表 + 条件判断 / middleware** 表达同等语义：

| Java SaRouter 特征 | Rust 等价做法 |
|---|---|
| `match("/user/**").check(...)` | `nest("/user", ...)` 或 path 匹配后 `check_*` |
| `match(a, b, c)` 多 path | 多个 `route` / `nest` |
| `notMatch("*.html")` | 静态资源单独挂载，不进鉴权 scope |
| `match(SaHttpMethod.GET)` | `routing::get(...)` 只注册 GET |
| `match(bool)` / lambda | `if` / middleware 内条件 |
| 连缀 `match().match().notMatch().check()` | 组合 path + method + `if` 条件后再 `check_*` |

概念示例（保留 Java 语义说明）：

``` rust
// 基础：匹配 /user/** 执行登录校验
async fn user_need_login() -> SaResult<()> {
    StpUtil::check_login()
}

// GET + /user/** 
async fn user_get_only() -> SaResult<()> {
    // 仅挂在 get 路由上即等价于 method 匹配
    StpUtil::check_login()
}

// 布尔条件
async fn when_logged() -> SaResult<&'static str> {
    if StpUtil::is_login()? {
        Ok("已登录分支")
    } else {
        Ok("未登录分支")
    }
}
```

Java 原文连缀样例（对照保留）：

``` java
SaRouter
	.match(SaHttpMethod.GET)
	.match("/admin/**")
	.match("/**/send/**")
	.notMatch("/**/*.js")
	.notMatch("/**/*.css")
	.check( /* 全部匹配成功才执行 */ );
```


### 4、提前退出匹配链

Java 使用 `SaRouter.stop()` 可以提前退出匹配链；`back()` 则停止匹配并直接向前端返回结果。

Rust 侧等价：

| Java | Rust |
|---|---|
| `SaRouter.stop()` | `return Ok(next.run(req).await)` 提前结束后续鉴权逻辑；或 `return` 跳出 middleware 链中后续 check |
| `SaRouter.back(body)` | 直接 `return Err(...)` / `return Response`，不再调用 `next` |

``` rust
async fn auth_chain_demo(path: &str) -> Result<&'static str, &'static str> {
    println!("进入1");
    println!("进入2");
    // 等价 stop：后续 check 不再执行，但仍可进入 handler
    if path.starts_with("/early") {
        return Ok("stopped-continue-handler");
    }
    println!("进入3");
    // 等价 back：直接返回给前端，不进入后续业务
    if path == "/user/back" {
        return Err("要返回到前端的内容");
    }
    Ok("ok")
}
```

stop() 与 back() 函数的区别在于：
- `SaRouter.stop()` 会停止匹配，进入 Controller。
- `SaRouter.back()` 会停止匹配，直接返回结果到前端。


### 5、使用 free 打开一个独立的作用域

Java `free()` 的作用是：打开一个独立的作用域，使内部的 `stop()` 不再一次性跳出整个 Auth 函数，而是仅仅跳出当前 free 作用域。

Rust 侧可用 **嵌套函数 / 独立 middleware / 独立 scope** 表达：

``` rust
async fn free_scope() -> SaResult<()> {
    // free 内部
    // match /a/**
    // match /b/** → 此处 return Ok(()) 仅结束本 scope
    // match /c/**
    Ok(())
}

async fn after_free() -> SaResult<()> {
    free_scope().await?;
    StpUtil::check_login()?; // free 之外继续
    Ok(())
}
```


### 6、使用注解忽略掉路由拦截校验

我们可以使用 `#[sa_ignore]` 注解，忽略掉路由拦截认证：

1、先配置好了拦截规则（按模块 check_permission，见第 2 节）。

2、然后在 handler 上添加忽略校验的注解：

``` rust
#[sa_ignore]
async fn get_list() -> &'static str {
    println!("------------ 访问进来方法");
    "ok"
}
```

请求将会跳过拦截器的校验，直接进入方法中。

> [!WARNING| label:注意点]
> 注解 `#[sa_ignore]` 的忽略效果主要针对框架 Layer / 注解鉴权生效；对你完全手写的自定义 middleware，需自行判断是否尊重该标记。




### 7、关闭注解校验

Java `SaInterceptor` 注册后默认打开注解校验，可用 `isAnnotation(false)` 关闭。

Rust 侧：若仅做路由拦截、不使用 `#[sa_check_*]`，则只需注册 Layer / `require_login`，不必启用注解宏扫描；需要关闭时不要挂载注解相关处理器即可。

你也可以使用「认证前置」逻辑（对应 Java `setBeforeAuth`）：

``` rust
async fn before_auth_then_auth(
    req: axum::extract::Request,
    next: axum::middleware::Next,
) -> axum::response::Response {
    println!("2"); // beforeAuth
    // 注解鉴权（若启用）在此之后
    println!("1"); // auth
    next.run(req).await
}
```

如上代码，先执行 2，再执行注解鉴权，再执行 1；若 beforeAuth 里直接返回响应（等价 `SaRouter.stop()`），将跳过后续鉴权环节。



---

<a class="case-btn" href="https://github.com/easy-4-rust/sa-token-rs/tree/main/crates/sa-token-demo/sa-token-demo-axum"
	target="_blank">
	本章代码示例：Sa-Token-Rs 路由拦截鉴权 —— [ sa-token-demo-axum ]
</a>
<a class="dt-btn" href="https://www.wenjuan.ltd/s/rY7VFv/" target="_blank">本章小练习：Sa-Token 基础 - 路由拦截鉴权，章节测试</a>
