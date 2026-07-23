# Http Basic 认证 

> Sa-Token → Sa-Token-Rs。对应模块：`SaHttpBasicUtil` / `SaHttpDigestUtil`；配置字段 `http_basic` / `http_digest`。

| Java | Rust |
|---|---|
| `SaHttpBasicUtil.check("sa:123456")` | `SaHttpBasicUtil::check_with_account("sa:123456")?` |
| `SaHttpBasicUtil.check()` | `SaHttpBasicUtil::check()?`（读全局 `http_basic`） |
| `@SaCheckHttpBasic` | `SaCheckHttpBasicMeta` / `#[sa_check_http_basic]` |
| `SaHttpDigestUtil.check("sa", "123456")` | `SaHttpDigestUtil::check_with_account("sa", "123456")?` |
| `SaServletFilter` + Basic | axum `SaTokenLayer` + handler/`SaRouter` 内调用 check |

Http Basic 是 http 协议中最基础的认证方式，其有两个特点：
- 简单、易集成。
- 功能支持度低。

在 Sa-Token-Rs 中使用 Http Basic 认证非常简单，只需调用几个简单的方法 

--- 

### 1、启用 Http Basic 认证 

首先我们在一个接口中，调用 Http Basic 校验：
``` rust
use axum::Json;
use sa_token::prelude::*;
use sa_token_core::http_auth::SaHttpBasicUtil;
use serde_json::{json, Value};

/// Basic 认证示例接口
async fn test3() -> SaResult<Json<Value>> {
    SaHttpBasicUtil::check_with_account("sa:123456")?;
    // ... 其它代码
    Ok(Json(json!({ "code": 200, "msg": "ok" })))
}
```

全局异常处理（axum 示例，对应 Java `@RestControllerAdvice`）：
``` rust
use axum::Json;
use axum::response::IntoResponse;
use sa_token_core::exception::SaTokenException;
use serde_json::json;

/// 将 Sa-Token 异常转为 JSON 响应
async fn handle_sa_error(err: SaTokenException) -> impl IntoResponse {
    eprintln!("{err:?}");
    Json(json!({ "code": 500, "msg": err.to_string() }))
}
```

然后我们访问这个接口时，浏览器会强制弹出一个表单：

<img class="s-w-sh" src="/big-file/doc/up/sa-basic.png" alt="sa-basic.png">


当我们输入账号密码后 `（sa / 123456）`，才可以继续访问数据：

<img class="s-w-sh" src="/big-file/doc/up/sa-basic-ok.png" alt="sa-basic-ok.png">


### 2、其它启用方式 
``` rust
use sa_token_core::http_auth::SaHttpBasicUtil;
use sa_token_core::annotation::sa_check_http_basic::SaCheckHttpBasicMeta;
use sa_token_core::router::SaRouter;

// 对当前会话进行 Http Basic 校验，账号密码为全局配置的值
// （例如：SaTokenConfig.http_basic = "sa:123456"）
SaHttpBasicUtil::check()?;

// 对当前会话进行 Http Basic 校验，账号密码为：`sa / 123456`
SaHttpBasicUtil::check_with_account("sa:123456")?;

// 以注解 / 元数据方式启用 Http Basic 校验
let _meta = SaCheckHttpBasicMeta::with_account("sa", "123456");
// 或在 handler 入口直接 check_with_account

// 在全局 Layer / 过滤器中启用 Basic 认证
let _ = SaRouter::match_paths(&["/test/**"]).check(|_| {
    SaHttpBasicUtil::check_with_account("sa:123456")
});
// Router::new().layer(SaTokenLayer::new()) ...
```

### 3、URL 认证 
除了访问后再输入账号密码外，我们还可以在 URL 中直接拼接账号密码通过 Basic 认证，例如：
``` url
http://sa:123456@127.0.0.1:8081/test/test3
```


### 4、Http Digest 认证 

Http Digest 认证是 Http Basic 认证的升级版，Http Digest 在提交请求时不会使用明文方式传输认证信息，而是使用一定的规则加密后提交。
不过对于开发者来讲，开启 Http Digest 认证校验的流程与 Http Basic 认证基本是一致的。

``` rust
use axum::Json;
use sa_token_core::http_auth::SaHttpDigestUtil;
use sa_token_core::annotation::sa_check_http_digest::SaCheckHttpDigestMeta;
use serde_json::{json, Value};

/// 测试 Http Digest 认证   浏览器访问： http://localhost:8081/test/testDigest
async fn test_digest() -> sa_token_core::exception::SaResult<Json<Value>> {
    SaHttpDigestUtil::check_with_account("sa", "123456")?;
    Ok(Json(json!({ "code": 200 })))
}

// 使用注解 / 元数据方式开启 Http Digest 认证
let _meta = SaCheckHttpDigestMeta::with_account("sa", "123456");

// 对当前会话进行 Http Digest 校验，账号密码为全局配置的值
// （例如：SaTokenConfig.http_digest = "sa:123456"）
SaHttpDigestUtil::check()?;
```

与上面的 Http Basic 认证一致，在访问这个路由时，浏览器会强制弹出一个表单，客户端输入正确的账号密码后即可通过校验。

同样的，Http Digest 也支持在浏览器访问接口时直接使用 @ 符号拼接账号密码信息，使客户端直接通过校验。

``` url
http://sa:123456@127.0.0.1:8081/test/testDigest
```



--- 

<a class="case-btn" href="https://github.com/sa-token-rust/sa-token-rs"
	target="_blank">
	本章代码示例：Sa-Token-Rs Http Basic 认证 —— [ SaHttpBasicUtil / SaHttpDigestUtil ]
</a>
