# 注解鉴权

> **⚠️ 文档适配状态**：本文档为 Java 官方文档的 Rust 移植版。注解宏来自
> `sa-token-derive` crate，由 `sa-token-web-axum` 的 `SaTokenLayer` 在请求
> 到达 handler 前自动展开，行为与 Java `@SaCheckXxx` 一致。

有同学表示：尽管使用代码鉴权非常方便，但是我仍希望把鉴权逻辑和业务逻辑分离开来，我可以使用注解鉴权吗？当然可以！<br>

注解鉴权 —— 优雅的将鉴权与业务代码分离！

- `#[sa_check_login]`：登录校验 —— 只有登录之后才能进入该方法。
- `#[sa_check_role("admin")]`：角色校验 —— 必须具有指定角色标识才能进入该方法。
- `#[sa_check_permission("user:add")]`：权限校验 —— 必须具有指定权限才能进入该方法。
- `#[sa_check_safe]`：二级认证校验 —— 必须二级认证之后才能进入该方法。
- `#[sa_check_http_basic]`：HttpBasic 校验 —— 只有通过 HttpBasic 认证后才能进入该方法。
- `#[sa_check_http_digest]`：HttpDigest 校验 —— 只有通过 HttpDigest 认证后才能进入该方法。
- `#[sa_check_disable("comment")]`：账号服务封禁校验 —— 校验当前账号指定服务是否被封禁。
- `#[sa_check_sign]`：API 签名校验 —— 用于跨系统的 API 签名参数校验。
- `#[sa_ignore]`：忽略校验 —— 表示被修饰的方法或结构体无需进行注解鉴权和路由拦截器鉴权。

Sa-Token-Rs 使用 axum 的 `Layer` 完成注解鉴权功能，为了不带来不必要的性能负担，注解鉴权默认处于关闭状态。
因此，为了使用注解鉴权，<green>**你必须手动将 `SaTokenLayer` 注册到你项目的 `Router` 上**</green>。


### 1、注册 Layer

以 axum 项目为例，新建 `auth_layer.rs`：

``` rust
use axum::Router;
use sa_token_axum::layer::SaTokenLayer;

pub fn router() -> Router {
    let app = Router::new()
        .route("/info", axum::routing::get(info))
        .route("/add",  axum::routing::post(add));
    // 注册 Sa-Token 拦截 Layer，打开注解式鉴权功能 
    app.layer(SaTokenLayer::new())
}
```

> 注意：在 `axum 0.7+` 中，若通过 `tower::ServiceBuilder` 组合多个 Layer，
> 必须将 `SaTokenLayer` 放在最外层才能读取到请求 Cookie / Header 中的 token。


### 2、使用注解鉴权

然后我们就可以愉快的使用注解鉴权了：

``` rust
use axum::routing::{get, post};
use sa_token_axum::SaTokenLayer;
use sa_token_derive::*;

// 登录校验：只有登录之后才能进入该方法 
#[sa_check_login]
#[axum::debug_handler]
async fn info() -> &'static str {
    "查询用户信息"
}

// 角色校验：必须具有指定角色才能进入该方法 
#[sa_check_role("super-admin")]
#[axum::debug_handler]
async fn add() -> &'static str {
    "用户增加"
}

// 权限校验：必须具有指定权限才能进入该方法 
#[sa_check_permission("user-add")]
#[axum::debug_handler]
async fn add_perm() -> &'static str {
    "用户增加"
}

// 二级认证校验：必须二级认证之后才能进入该方法 
#[sa_check_safe]
#[axum::debug_handler]
async fn add_safe() -> &'static str {
    "用户增加"
}

// Http Basic 校验：只有通过 Http Basic 认证后才能进入该方法 
#[sa_check_http_basic(account = "sa:123456")]
#[axum::debug_handler]
async fn add_basic() -> &'static str {
    "用户增加"
}

// Http Digest 校验：只有通过 Http Digest 认证后才能进入该方法 
#[sa_check_http_digest(value = "sa:123456")]
#[axum::debug_handler]
async fn add_digest() -> &'static str {
    "用户增加"
}

// 校验当前账号是否被封禁 comment 服务，如果已被封禁会抛出异常，无法进入 handler 
#[sa_check_disable("comment")]
#[axum::debug_handler]
async fn send() -> &'static str {
    "查询用户信息"
}
```

注：以上注解都可以加在 `impl` 块 / `Router` 子路由组上，代表为这一组所有方法进行鉴权（搭配 `Router::route_service` 使用）。


### 3、设定校验模式

`#[sa_check_role]` 与 `#[sa_check_permission]` 注解可设置校验模式，例如：

``` rust
// 注解式鉴权：只要具有其中一个权限即可通过校验 
#[sa_check_permission(
    value = ["user-add", "user-all", "user-delete"],
    mode = SaMode::OR,
)]
#[axum::debug_handler]
async fn at_jur_or() -> &'static str {
    "用户信息"
}
```

`mode` 有两种取值：
- `SaMode::AND`：标注一组权限，会话必须全部具有才可通过校验。
- `SaMode::OR`：标注一组权限，会话只要具有其一即可通过校验。


### 4、角色权限双重 "or 校验"

假设有以下业务场景：一个接口在具有权限 `user.add` 或角色 `admin` 时可以调通。怎么写？

``` rust
// 角色权限双重 "or 校验"：具备指定权限或者指定角色即可通过校验
#[sa_check_permission(value = "user.add", or_role = "admin")]
#[axum::debug_handler]
async fn user_add() -> &'static str {
    "用户信息"
}
```

`or_role` 字段代表权限校验未通过时的次要选择，两者只要其一校验成功即可进入请求方法，其有三种写法：

- 写法一：`or_role = "admin"`，代表需要拥有角色 `admin`。
- 写法二：`or_role = ["admin", "manager", "staff"]`，代表具有三个角色其一即可。
- 写法三：`or_role = ["admin,manager,staff"]`，代表必须同时具有三个角色。


### 5、忽略认证

使用 `#[sa_ignore]` 可表示一个接口忽略认证：

``` rust
use axum::routing::get;
use axum::Router;
use sa_token_derive::*;

#[sa_check_login]
#[derive(Default)]
struct TestController;

#[sa_ignore]
async fn get_list() -> &'static str {
    // ...
    "ok"
}

pub fn router() -> Router {
    Router::new()
        .route("/getList", get(get_list))
        // ...其它需要登录后才能访问的方法
}
```

如上代码表示：`TestController` 中的所有方法都需要登录后才可以访问，但是 `get_list` 接口可以匿名游客访问。

- `#[sa_ignore]` 修饰函数时代表这个方法可以被游客访问，修饰结构体 / Router 子组时代表这个作用域内的所有接口都可以游客访问。
- `#[sa_ignore]` 具有最高优先级，当它与其它鉴权注解一起出现时，其它鉴权注解都将被忽略。
- `#[sa_ignore]` 同样可以忽略掉 Sa-Token Layer 中的路由鉴权，在下面的 [路由拦截鉴权] 章节中我们会讲到。


### 6、批量注解鉴权

使用 `#[sa_check_or]` 表示批量注解鉴权：

``` rust
// 在 `#[sa_check_or]` 中可以指定多个注解，只要当前会话满足其中一个注解即可通过验证
#[sa_check_or(
    login      = SaCheckLogin::new(),
    role       = SaCheckRole::new("admin"),
    permission = SaCheckPermission::new("user.add"),
    safe       = SaCheckSafe::new("update-password"),
    http_basic = SaCheckHttpBasic::new(account = "sa:123456"),
    disable    = SaCheckDisable::new("submit-orders"),
)]
#[axum::debug_handler]
async fn test() -> &'static str {
    // ...
    "ok"
}
```

每一项属性都可以写成数组形式，例如：

``` rust
// 当前客户端只要有 [ login 账号登录] 或者 [user 账号登录] 其一，就可以通过验证
//      注意：`kind = LoginKind::Login` 和 `kind = LoginKind::User` 是多账号模式章节的扩展属性，
//      此处你可以先略过这个知识点。
#[sa_check_or(
    login = [
        SaCheckLogin::new(kind = LoginKind::Login),
        SaCheckLogin::new(kind = LoginKind::User),
    ],
)]
#[axum::debug_handler]
async fn test_multi() -> &'static str {
    // ...
    "ok"
}
```

疑问：既然有了 `#[sa_check_or]`，为什么没有与之对应的 `#[sa_check_and]` 呢？

因为当你写多个注解时，其天然就是 `and` 校验关系，例如：

``` rust
// 当你在一个方法上写多个注解鉴权时，其默认就是要满足所有注解规则后，才可以进入方法
// 只要有一个不满足，就会抛出异常
#[sa_check_login]
#[sa_check_role("admin")]
#[sa_check_permission("user.add")]
#[axum::debug_handler]
async fn test_and() -> &'static str {
    // ...
    "ok"
}
```


使用 `append` 字段追加抓取扩展 crate 里的注解，例如：

``` rust
// 测试：只有通过登录校验，或者提供了正确的 ApiKey，才可以进入方法
#[sa_check_or(
    login  = SaCheckLogin::new(),
    append = [SaCheckApiKey::marker()],
)]
#[sa_check_api_key]
#[axum::debug_handler]
async fn test_api_key() -> &'static str {
    // ...
    "ok"
}
```


### 7、扩展阅读

- 在业务逻辑层使用鉴权宏：[AOP 注解鉴权](/plugin/aop-at)
- 制作自定义鉴权宏注入到框架：[自定义注解](/fun/custom-annotations)


---

<a class="case-btn" href="https://github.com/your-org/sa-token-rs/blob/main/crates/sa-token-demo-axum/src/at_check_controller.rs"
	target="_blank">
	本章代码示例：Sa-Token-Rs 注解鉴权 —— [ at_check_controller.rs ]
</a>
<a class="dt-btn" href="https://www.wenjuan.ltd/s/ARJvIbA/" target="_blank">本章小练习：Sa-Token 基础 - 注解鉴权，章节测试</a>