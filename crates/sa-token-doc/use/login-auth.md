# 登录认证

> 本文保留 Java 原文章节结构与说明文字，示例已改为 **Sa-Token-Rs（Rust）**。

| Java | Rust |
|---|---|
| `StpUtil.login(10001)` | `StpUtil::login("10001")?` |
| `StpUtil.isLogin()` | `StpUtil::is_login()?` |
| `StpUtil.checkLogin()` | `StpUtil::check_login()?` |
| `SaResult.ok(...)` | `Json(serde_json::json!({ ... }))` 等 |
| `@RequestMapping` | axum `Router` / actix `App` |

---


### 1、开始登录

一个完整的登录认证包含哪些步骤？让我们代入用户视角：在打开 网站/APP 后，用户的操作流程大致可以概括为：
1. 打开 网站/APP，进入登录页。
2. 输入 账号+密码 进行登录。
3. 进入首页，进行业务相关操作。
4. 注销登录，关闭 网站/APP。

在整个流程中，Sa-Token-Rs 负责哪些部分呢？ 下图可以帮助你理解：

<img class="w-100" src="/big-file/doc/use/use-login-auth.svg" />

如上图所示：<green>**无论用户采用何种登录方式，本质上都是通过提交一定的认证信息，使系统可以定位到 Ta 的唯一标识 —— userId**</green>。

当我们拿到 userId 后，便可以调用框架提供的 API 进行登录：

``` rust
// 会话登录：参数填写要登录的账号 id，建议使用 &str（由业务侧把数字 id 转成字符串）
// 不可以传入复杂类型，如：结构体 User、Admin 等等
StpUtil::login("10001")?;
```

只此一句代码，便可以使会话登录成功。实际上，Sa-Token-Rs 在背后做了大量的工作，包括但不限于：

1. 检查此账号是否之前已有登录；
2. 为账号生成 Token 凭证与 Session 会话；
3. 记录 Token 活跃时间；
4. 通知全局侦听器，xx 账号登录成功；
5. 检查此账号登录数量是否已达上限；
6. 将 Token 注入到请求上下文；
7. 等等其它工作……

你暂时不需要完整了解完整过程，你只需要记住关键一点：<green>**Sa-Token-Rs 为这个账号创建了一个 token 凭证，且通过 Cookie 上下文返回给了前端**</green>。

所以一般情况下，我们的登录接口代码，会大致类似如下：

``` rust
use axum::{extract::Query, Json};
use sa_token::prelude::*;
use serde::Deserialize;
use serde_json::{json, Value};

#[derive(Deserialize)]
struct LoginQuery {
    name: String,
    pwd: String,
}

/// 会话登录接口 —— 对应 Java @RequestMapping("doLogin")
async fn do_login(Query(q): Query<LoginQuery>) -> Json<Value> {
    // 第一步：比对前端提交的账号名称、密码
    if q.name == "zhang" && q.pwd == "123456" {
        // 第二步：根据账号 id，进行登录
        let _ = StpUtil::login("10001");
        return Json(json!({ "code": 200, "msg": "登录成功" }));
    }
    Json(json!({ "code": 500, "msg": "登录失败" }))
}
```

如果你对以上代码阅读没有压力，你可能会注意到略显奇怪的一点：<green>**此处仅仅做了会话登录，但并没有主动向前端返回 token 信息。**</green>

是因为不需要吗？严格来讲是需要的，只不过 `StpUtil::login(id)` 方法利用了 Cookie 自动注入的特性，省略了你手写返回 token 的代码。

> [!TIP| label:Cookie 是什么？]
> 如果你对 Cookie 功能还不太了解，也不用担心，我们会在之后的 [ 前后端分离 ] 章节中详细的阐述 Cookie 功能，现在你只需要了解最基本的两点：
>
> - Cookie 可以从后端控制往浏览器中写入 token 值。
> - Cookie 会在前端每次发起请求时自动提交 token 值。
>
> 因此，在 Cookie 功能的加持下，我们可以仅靠 `StpUtil::login(id)` 一句代码就完成登录认证。
>
> 在浏览器打开 f12 控制台，即可看到被注入的 Cookie 值：
>
> <button class="show-img" img-src="/big-file/doc/use/sa-login-cookie-pre.png">加载演示图</button>


### 2、校验是否登录

对于一些登录之后才能访问的接口（例如：查询我的账号资料），我们通常的做法是增加一层接口校验：

- 如果校验通过，则：<green>正常返回数据。</green>
- 如果校验未通过，则：<red>抛出异常（返回 Err），告知其需要先进行登录。</red>

<img class="w-100" src="/big-file/doc/use/use-login-check.svg" />

<!-- <button class="show-img" img-src="/big-file/doc/use/g3--login-auth.gif">加载动态演示图</button> -->

使用以下方法判断当前会话是否已登录：

``` rust
// 判断当前会话是否已经登录，返回 Ok(true)=已登录，Ok(false)=未登录
StpUtil::is_login()?;

// 检验当前会话是否已经登录；已登录则 Ok(())，未登录则返回 Err（对应 Java NotLoginException）
StpUtil::check_login()?;
```

例如我们可以在接口内，根据是否登录返回不同的信息：

``` rust
/// 获取我的资料信息
async fn my_info() -> &'static str {
    if StpUtil::is_login().unwrap_or(false) {
        // ...
        "我的资料信息..."
    } else {
        "未登录，请先登录"
    }
}
```

或者在未登录时直接返回错误：

``` rust
/// 获取我的资料信息
async fn my_info() -> SaResult<&'static str> {
    StpUtil::check_login()?; // 如果当前未登录，这句会直接返回 Err
    Ok("我的资料信息")
}
```

配合全局错误处理，统一返回固定格式数据到前端：

``` rust
use axum::{http::StatusCode, response::IntoResponse, Json};
use serde_json::json;

/// 对应 Java @RestControllerAdvice + @ExceptionHandler(NotLoginException.class)
fn map_not_login(msg: impl ToString) -> impl IntoResponse {
    (
        StatusCode::UNAUTHORIZED,
        Json(json!({ "code": 401, "msg": msg.to_string() })),
    )
}
```

异常 / 错误 <red>`NotLogin`</red>（Java 中为 `NotLoginException`）代表当前会话暂未登录，可能的原因有很多：
- 前端没有提交 token。
- 前端提交的 token 是无效的。
- 前端提交的 token 已经过期。
- ……

可参照此篇：[未登录场景值](/fun/not-login-scene)，了解如何获取未登录的场景值。


### 3、会话查询

如果你想要获取当前登录的是谁：

``` rust
// 获取当前会话账号 id，如果未登录，则返回 Err（对应 NotLoginException）
StpUtil::get_login_id()?;

// 类似查询 API 还有：
StpUtil::get_login_id_as_string()?; // 获取当前会话账号 id，并转化为 String
StpUtil::get_login_id_as_i32()?;    // 转化为 i32
StpUtil::get_login_id_as_i64()?;    // 转化为 i64

// ---------- 以下方法可以指定未登录情形下返回的默认值 ----------

// 获取当前会话账号 id，如果未登录，则返回 None
StpUtil::get_login_id_default_null()?;
```

| Java | Rust |
|---|---|
| `StpUtil.getLoginId()` | `StpUtil::get_login_id()?` |
| `StpUtil.getLoginIdAsLong()` | `StpUtil::get_login_id_as_i64()?` |
| `StpUtil.getLoginIdDefaultNull()` | `StpUtil::get_login_id_default_null()?` |


### 4、token 查询

``` rust
// 获取当前会话的 token 值（Option）
StpUtil::get_token_value();

// 获取当前 StpLogic 的 token 名称
StpUtil::get_token_name();

// 获取指定 token 对应的账号 id，如果未登录，则返回 None
StpUtil::get_login_id_by_token("token_value")?;

// 获取当前会话剩余有效期（单位：s，返回 -1 代表永久有效）
StpUtil::get_token_timeout()?;

// 获取当前会话的 token 信息参数
StpUtil::get_token_info()?;
```

有关 `TokenInfo` 参数详解，请参考：[TokenInfo 参数详解](/fun/token-info)


### 5、会话注销

``` rust
// 当前会话注销登录
StpUtil::logout()?;
```


### 6、来个小测试，加深一下理解

新建 `login_controller` 模块，复制或手动敲出以下代码：

``` rust
use axum::{
    extract::Query,
    routing::{get, post},
    Json, Router,
};
use sa_token::prelude::*;
use serde::Deserialize;
use serde_json::{json, Value};

#[derive(Deserialize)]
struct LoginQuery {
    name: String,
    pwd: String,
}

/// 登录测试 —— 对应 Java LoginController
/// 测试登录  ---- http://localhost:8081/acc/doLogin?name=zhang&pwd=123456
async fn do_login(Query(q): Query<LoginQuery>) -> Json<Value> {
    // 此处仅作模拟示例，真实项目需要从数据库中查询数据进行比对
    if q.name == "zhang" && q.pwd == "123456" {
        let _ = StpUtil::login("10001");
        Json(json!({ "code": 200, "msg": "登录成功" }))
    } else {
        Json(json!({ "code": 500, "msg": "登录失败" }))
    }
}

/// 查询登录状态  ---- http://localhost:8081/acc/isLogin
async fn is_login() -> Json<Value> {
    Json(json!({
        "code": 200,
        "msg": format!("是否登录：{}", StpUtil::is_login().unwrap_or(false))
    }))
}

/// 查询 Token 信息  ---- http://localhost:8081/acc/tokenInfo
async fn token_info() -> Json<Value> {
    match StpUtil::get_token_info() {
        Ok(info) => Json(json!({ "code": 200, "data": format!("{info:?}") })),
        Err(e) => Json(json!({ "code": 500, "msg": e.to_string() })),
    }
}

/// 测试注销  ---- http://localhost:8081/acc/logout
async fn logout() -> Json<Value> {
    let _ = StpUtil::logout();
    Json(json!({ "code": 200 }))
}

/// 组装路由
pub fn router() -> Router {
    Router::new()
        .route("/acc/doLogin", get(do_login).post(do_login))
        .route("/acc/isLogin", get(is_login))
        .route("/acc/tokenInfo", get(token_info))
        .route("/acc/logout", get(logout))
}
```

完整可运行工程见：`crates/sa-token-demo/sa-token-demo-axum`。

---

<a class="case-btn" href="https://github.com/easy-4-rust/sa-token-rs/tree/main/crates/sa-token-demo/sa-token-demo-axum"
	target="_blank">
	本章代码示例：Sa-Token-Rs 登录认证 —— [ sa-token-demo-axum ]
</a>
<a class="dt-btn" href="https://www.wenjuan.ltd/s/UZBZJvb2ej/" target="_blank">本章小练习：Sa-Token 基础 - 登录认证，章节测试</a>
