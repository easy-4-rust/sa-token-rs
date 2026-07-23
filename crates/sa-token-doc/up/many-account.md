# 多账号认证
---

> Sa-Token → Sa-Token-Rs。推荐 Rust 侧用 **Kit 模式**（多个 `StpLogic`）而非复制整份门面代码。

| Java | Rust |
|---|---|
| `StpUtil` / `StpUserUtil` | `StpUtil` + `StpLogic::new("user")` / Kit |
| `new StpLogic("admin")` | `StpLogic::new("admin")` + `SaManager::put_stp_logic` |
| `@SaCheckLogin(type = "user")` | 宏若支持 type 则指定；否则对该逻辑实例显式 `check_login` |
| `SaInterceptor` + `SaRouter` | `SaTokenLayer` / 路由中间件内组合校验 |

### 1、需求场景
有的时候，我们会在一个项目中设计两套账号体系，比如一个电商系统的 `user表` 和 `admin表`，
在这种场景下，如果两套账号我们都使用 `StpUtil` 类的API进行登录鉴权，那么势必会发生逻辑冲突。

在 Sa-Token-Rs 中，这个问题的模型叫做：多账号体系认证。

要解决这个问题，我们必须有一个合理的机制将这两套账号的授权给区分开，让它们互不干扰才行。


### 2、演进思路
假如说我们的 user表 和 admin表 都有一个 id=10001 的账号，它们对应的登录代码：`StpUtil::login("10001")` 是一样的，
那么问题来了：在`StpUtil::get_login_id()`获取到的账号id如何区分它是User用户，还是Admin用户？

你可能会想到为他们加一个固定前缀，比如`StpUtil::login(&format!("User_{}", 10001))`、`StpUtil::login(&format!("Admin_{}", 10001))`，这样确实是可以解决问题的，
但是同样的：你需要在`get_login_id`时再裁剪掉相应的前缀才能获取真正的账号id，这样一增一减就让我们的代码变得无比啰嗦。

那么，有没有从框架层面支持的，更优雅的解决方案呢？


### 3、解决方案

前面几篇介绍的api调用，都是经过 StpUtil 的各种静态方法进行授权认证，
而如果我们深入它的源码就会发现，此类并没有任何代码逻辑，唯一做的事就是对成员变量`stp_logic`的各个API包装一下进行转发。

这样做有两个好处:
- StpLogic 的行为可以按需扩展。
- 在构造方法时随意传入一个不同的 `login_type`，就可以再造一套账号登录体系。


### 4、操作示例

比如说，对于原生`StpUtil`类，我们只做`admin账号`权限认证，而对于`user账号`，我们则：
1. 新建一个新的权限认证门面，比如： `StpUserUtil`。
2. 内部持有 `login_type = "user"` 的 `StpLogic`。
3. 启动时 `SaManager::put_stp_logic` 注册。

``` rust
use std::sync::Arc;
use sa_token::prelude::*;

/// User 账号体系门面（对应 Java StpUserUtil）
pub struct StpUserUtil;

impl StpUserUtil {
    /// 账号体系标识
    pub const TYPE: &'static str = "user";

    fn logic() -> Arc<StpLogic> {
        SaManager::get_stp_logic(Self::TYPE).expect("StpUserLogic not initialized")
    }

    pub fn login(id: &str) -> SaResult<()> {
        Self::logic().login(id)
    }

    pub fn check_login() -> SaResult<()> {
        Self::logic().check_login()
    }

    // 其它 API 同样转发到 Self::logic() ...
}

/// 启动时注册
fn init_user_logic() {
    SaManager::put_stp_logic(Arc::new(StpLogic::new(StpUserUtil::TYPE)));
}
```

4、接下来就可以像调用`StpUtil`一样调用 `StpUserUtil`了，这两套账号认证的逻辑是完全隔离的。例如：

``` rust
// 凡是在 StpUtil 上有的方法，都可以在 StpUserUtil 上封装
StpUserUtil::login("10001")?;    // 在当前会话以10001账号进行登录
StpUserUtil::check_login()?;     // 校验当前账号是否以 User 身份进行登录
// StpUserUtil::get_session()?;
// StpUserUtil::check_permission("xx")?;
```


### 5、Kit模式
如果你觉得 “复制门面代码” 的方式繁琐不够优雅，这里还有另一种方案：建立一个 `StpKit` 门面，声明所有的 `StpLogic` 引用：
``` rust
use std::sync::Arc;
use once_cell::sync::Lazy;
use sa_token::prelude::*;

/// StpLogic 门面，管理项目中所有的账号体系
pub struct StpKit;

impl StpKit {
    /// 默认原生会话对象（login）
    pub fn default_logic() -> Arc<StpLogic> {
        StpUtil::stp_logic()
    }

    /// Admin 会话对象
    pub fn admin() -> Arc<StpLogic> {
        SaManager::get_stp_logic("admin").expect("admin StpLogic missing")
    }

    /// User 会话对象
    pub fn user() -> Arc<StpLogic> {
        SaManager::get_stp_logic("user").expect("user StpLogic missing")
    }
}

fn init_kit() {
    SaManager::put_stp_logic(Arc::new(StpLogic::new("admin")));
    SaManager::put_stp_logic(Arc::new(StpLogic::new("user")));
}
```

在需要登录、权限认证的地方：
``` rust
// 在当前会话进行 Admin 账号登录
StpKit::admin().login("10001")?;

// 在当前会话进行 User 账号登录
StpKit::user().login("10001")?;

// 检测当前会话是否以 Admin 账号登录，并具有 article:add 权限
StpKit::admin().check_permission("article:add")?;

// 检测当前会话是否以 User 账号登录，并通过了二级认证
StpKit::user().check_safe()?;

// 获取当前 User 会话的 Session 对象，并进行写值操作
let mut session = StpKit::user().get_session()?;
session.set("name", serde_json::json!("zhang"));
```


### 6、在多账户模式下使用注解鉴权
框架默认的注解鉴权 如`#[sa_check_login]` 通常只针对原生`StpUtil`进行鉴权。

例如，我们在一个方法上加上`#[sa_check_login]`，这个注解只会放行通过`StpUtil::login(id)`进行登录的会话，
而对于通过`StpUserUtil::login(id)`进行登录的会话，则可能始终不会通过校验。

那么如何告诉注解要鉴别的是哪套账号的登录会话呢？

``` rust
// 若宏支持 type 参数（以实现为准）：
// #[sa_check_login(type = "user")]
// 否则在 handler 内显式：
async fn info() -> SaResult<&'static str> {
    StpUserUtil::check_login()?;
    Ok("查询用户信息")
}
```

Java 对照：`@SaCheckLogin(type = StpUserUtil.TYPE)`；`@SaCheckRole` / `@SaCheckPermission` 同理。


### 7、使用注解合并简化代码
交流群里有同学反应，虽然可以根据 `@SaCheckLogin(type = "user")` 指定账号类型，但几十上百个注解都加上这个的话，还是有些繁琐。

Java 侧可通过 Spring `AnnotatedElementUtils` + 自定义组合注解实现合并。Rust 的 proc-macro 生态也可自定义属性宏（如 `#[sa_user_check_login]`），效果等同于指定 type。

Java 对照步骤保留如下，便于迁移：

1. 重写注解处理器（Java）/ 编写自定义 derive（Rust）
2. 自定义注解 `@SaUserCheckLogin` / `#[sa_user_check_login]`
3. 在方法上使用自定义注解

完整 Java 示例见 [Java 原版文档](https://sa-token.cc) 与 Gitee 注解合并示例。

> [!TIP| label:自定义注解方案]
> 除了注解合并方案，这里还有一份自定义注解方案，参考：[自定义注解](/fun/custom-annotations)



### 8、同端多登陆
假设我们不仅需要在后台同时集成两套账号，我们还需要在一个客户端同时登陆两套账号（业务场景举例：一个APP中可以同时登陆商家账号和用户账号）。

如果我们不做任何特殊处理的话，在客户端会发生`token覆盖`，新登录的 token 会覆盖掉旧登录的 token 从而导致旧登录失效。

具体表现大致为：在一个浏览器登录商家账号后，再登录用户账号，然后商家账号的登录态就会自动失效。

那么如何解决这个问题？很简单，我们只要为 User 体系使用不同的 `token_name` 即可，参考示例如下：

``` rust
use std::sync::Arc;
use sa_token::prelude::*;

fn init_user_with_distinct_token_name() {
    let mut logic = StpLogic::new("user");
    // 若 StpLogic 支持 set_config，为其指定独立 token_name，避免与默认 satoken 冲突
    logic.set_config(Arc::new(SaTokenConfig {
        token_name: "satoken-user".into(),
        ..Default::default()
    }));
    SaManager::put_stp_logic(Arc::new(logic));
}
```

再次调用 User 体系登录时，token 的名称将不再是 `satoken`，而是 `satoken-user`，这样就不会在客户端发生 token 的相互覆盖了。


### 9、不同体系不同 SaTokenConfig 配置
如果自定义的 User 体系需要使用不同 SaTokenConfig 对象, 也很简单：

``` rust
use std::sync::Arc;
use sa_token::prelude::*;

fn set_sa_token_config() {
    let config1 = Arc::new(SaTokenConfig {
        token_name: "satoken1".into(),
        timeout: 1000,
        token_style: SaTokenStyle::Random64,
        ..Default::default()
    });
    // StpUtil 默认 logic
    StpUtil::stp_logic().set_config(config1);

    let config2 = Arc::new(SaTokenConfig {
        token_name: "satoken2".into(),
        timeout: 2000,
        token_style: SaTokenStyle::Tik,
        ..Default::default()
    });
    SaManager::get_stp_logic("user")
        .expect("user logic")
        .set_config(config2);
}
```


### 10、多账号体系混合鉴权
QQ群中经常有小伙伴提问：在多账号体系下，怎么在拦截器 / Layer 中给一个接口登录鉴权？

其实这个问题，主要是靠你的业务需求来决定，以后台 Admin 账号和前台 User 账号为例：

``` rust
use axum::Router;
use sa_token_web_axum::SaTokenLayer;

/// 示意：在路由中间件 / Layer 回调中按业务组合校验
fn build_router() -> Router {
    // 伪代码：实际以 SaRouter / Layer 钩子 API 为准
    // /art/getInfo —— 仅 Admin：StpUtil::check_login()
    // /art/getInfo —— 仅 User：StpUserUtil::check_login()
    // 同时要求两者登录：两者都 check_login
    // 任意一个登录：is_login 或关系
    Router::new().layer(SaTokenLayer::new())
}
```

Java 对照（SaInterceptor）：
``` java
SaRouter.match("/art/getInfo").check(r -> StpUtil.checkLogin());
SaRouter.match("/art/getInfo").check(r -> StpUserUtil.checkLogin());
// 同时 / 任意一个 —— 见 Java 原版逻辑
```


### 11、在一个接口里获取是哪个体系的账号正在登录

可以分别用两个体系的 `is_login()` 方法去判断，哪个返回 true 就代表正在登录哪个体系

``` rust
async fn test2() -> SaResult<()> {
    let mut login_type = String::new();

    if StpUtil::is_login()? {
        login_type = StpUtil::TYPE.to_string();
    }
    if StpUserUtil::logic().is_login()? {
        login_type = StpUserUtil::TYPE.to_string();
    }

    println!("当前登录的 loginType：{login_type}");
    Ok(())
}
```

请注意此处可能出现的两种边际情况：
- 两个 if 均返回 false：代表客户端在两个账号体系都没有登录。
- 两个 if 均返回 true：代表客户端在两个账号体系都登录了。


### 12、注意点：运行时不可更改 LoginType
在 Q群 解决问题时，发现有些同学会写出类似下列形式的代码：

``` rust
StpUtil::login("10001")?;
// 错误：不要在运行时改 login_type
// StpUtil::stp_logic().set_login_type("user");
StpUtil::get_session()?.set("name", serde_json::json!("zhangsan"));
```

这是一种错误写法：LoginType 不可在运行时更改，只能在项目启动时指定。一旦项目启动成功后再修改 LoginType ，就会造成线程安全问题和严重的逻辑问题。
