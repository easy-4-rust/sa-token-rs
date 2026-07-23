# 注解鉴权

> 保留 Java 原文章节结构；`@SaCheckXxx` → `#[sa_check_xxx]`。

| Java | Rust |
|---|---|
| `@SaCheckLogin` | `#[sa_check_login]` |
| `@SaCheckRole("admin")` | `#[sa_check_role("admin")]` |
| `@SaCheckPermission("user:add")` | `#[sa_check_permission("user:add")]` |
| `@SaIgnore` | `#[sa_ignore]` |
| `SaInterceptor` | `SaTokenLayer`（axum） |


### 注解鉴权

有同学表示：尽管使用代码鉴权非常方便，但是我仍希望把鉴权逻辑和业务逻辑分离开来，我可以使用注解鉴权吗？当然可以！<br>

注解鉴权 —— 优雅的将鉴权与业务代码分离！

- `#[sa_check_login]`: 登录校验 —— 只有登录之后才能进入该方法。
- `#[sa_check_role("admin")]`: 角色校验 —— 必须具有指定角色标识才能进入该方法。
- `#[sa_check_permission("user:add")]`: 权限校验 —— 必须具有指定权限才能进入该方法。
- `#[sa_check_safe]`: 二级认证校验 —— 必须二级认证之后才能进入该方法。
- `@SaCheckHttpBasic` / `#[sa_check_http_basic]`（若已提供）: HttpBasic校验 —— 只有通过 HttpBasic 认证后才能进入该方法。
- `@SaCheckHttpDigest` / `#[sa_check_http_digest]`（若已提供）: HttpDigest校验 —— 只有通过 HttpDigest 认证后才能进入该方法。
- `#[sa_check_disable("comment")]`：账号服务封禁校验 —— 校验当前账号指定服务是否被封禁。
- `@SaCheckSign` / `#[sa_check_sign]`（若已提供）：API 签名校验 —— 用于跨系统的 API 签名参数校验。
- `#[sa_ignore]`：忽略校验 —— 表示被修饰的方法或类型无需进行注解鉴权和路由拦截器鉴权。

Sa-Token-Rs 使用 Web Layer / 中间件完成注解鉴权功能，为了不为项目带来不必要的性能负担，相关能力需要显式接入。
因此，为了使用注解鉴权，<green>**你必须手动将 Sa-Token-Rs 的 Layer / 中间件注册到你项目中**</green>。


### 1、注册拦截器（Layer）

以 axum 项目为例（对应 Java SpringBoot + `SaInterceptor`）：

``` rust
use axum::Router;
use sa_token_web_axum::SaTokenLayer;

/// 注册 Sa-Token-Rs Layer，打开注解式鉴权功能
fn build_router() -> Router {
    Router::new()
        // .route(...)
        .layer(SaTokenLayer::new())
}
```

保证启动时完成 `SaManager` 初始化（config / dao / stp_logic）即可。

<!-- 注意：具体宏展开与 Layer 协作细节以实现为准，可参考 sa-token-demo-axum -->


### 2、使用注解鉴权

然后我们就可以愉快的使用注解鉴权了：

``` rust
/// 登录校验：只有登录之后才能进入该方法
#[sa_check_login]
async fn info() -> &'static str {
    "查询用户信息"
}

/// 角色校验：必须具有指定角色才能进入该方法
#[sa_check_role("super-admin")]
async fn add_by_role() -> &'static str {
    "用户增加"
}

/// 权限校验：必须具有指定权限才能进入该方法
#[sa_check_permission("user-add")]
async fn add_by_permission() -> &'static str {
    "用户增加"
}

/// 二级认证校验：必须二级认证之后才能进入该方法
#[sa_check_safe]
async fn add_safe() -> &'static str {
    "用户增加"
}

/// 校验当前账号是否被封禁 comment 服务
#[sa_check_disable("comment")]
async fn send() -> &'static str {
    "查询用户信息"
}
```

Java 对照（节选）：

``` java
@SaCheckLogin
@RequestMapping("info")
public String info() { return "查询用户信息"; }
```

注：以上注解都可以加在类型 / 模块上（视宏支持范围），代表为相关方法进行鉴权。


### 3、设定校验模式

`#[sa_check_role]` 与 `#[sa_check_permission]` 可设置校验模式（属性名以 `sa-token-derive` 为准）。语义对照：

``` rust
// OR：具有其中一个权限即可 —— 也可用代码 API
async fn at_jur_or() -> SaResult<&'static str> {
    StpUtil::check_permission_or(&["user-add", "user-all", "user-delete"])?;
    Ok("用户信息")
}
```

mode 有两种取值（语义）：
- **AND**：标注一组权限，会话必须全部具有才可通过校验。
- **OR**：标注一组权限，会话只要具有其一即可通过校验。


### 4、角色权限双重 “or校验”

假设有以下业务场景：一个接口在具有权限 `user.add` 或角色 `admin` 时可以调通。怎么写？

``` rust
/// 具备指定权限或者指定角色即可通过校验
async fn user_add() -> SaResult<&'static str> {
    if StpUtil::has_permission("user.add")? || StpUtil::has_role("admin")? {
        return Ok("用户信息");
    }
    StpUtil::check_permission("user.add")?; // 失败时抛出权限错误
    Ok("用户信息")
}
```

Java 中 `orRole` 字段的三种写法，在 Rust 中可用 `has_role` / `has_role_or` / `check_role_and` 组合表达。


### 5、忽略认证

使用 `#[sa_ignore]` 可表示一个接口忽略认证：

``` rust
#[sa_check_login]
mod guarded {
    // ... 其它方法

    /// 此接口加上了 #[sa_ignore] 可以游客访问
    #[sa_ignore]
    async fn get_list() -> &'static str {
        "ok"
    }
}
```

如上代码表示：受 `#[sa_check_login]` 约束的方法需要登录后才可以访问，但是带 `#[sa_ignore]` 的接口可以匿名游客访问。

- `#[sa_ignore]` 修饰方法时代表这个方法可以被游客访问，修饰类型时代表相关接口都可以游客访问。
- `#[sa_ignore]` 具有最高优先级，当与其它鉴权注解一起出现时，其它鉴权注解都将被忽略。
- `#[sa_ignore]` 同样可以忽略掉路由拦截鉴权，在下面的 [路由拦截鉴权] 章节中我们会讲到。



### 6、批量注解鉴权

使用 `@SaCheckOr`（Java）表示批量注解鉴权；Rust 侧可用多个条件的 OR 组合或宏（若已提供 `sa_check_or`）：

``` rust
/// 满足登录、角色、权限等其一即可（示意）
async fn test_or() -> SaResult<&'static str> {
    if StpUtil::is_login()?
        || StpUtil::has_role("admin")?
        || StpUtil::has_permission("user.add")?
    {
        return Ok("ok");
    }
    StpUtil::check_login()?;
    Ok("ok")
}
```

当你写多个鉴权注解 / 多次 `check_*` 时，其天然就是 **AND** 校验关系，例如：

``` rust
#[sa_check_login]
#[sa_check_role("admin")]
#[sa_check_permission("user.add")]
async fn test_and() -> &'static str {
    "ok"
}
```


### 7、扩展阅读

- 在业务逻辑层使用鉴权注解：[AOP注解鉴权](/plugin/aop-at)

- 制作自定义鉴权注解注入到框架：[自定义注解](/fun/custom-annotations)


---

<a class="case-btn" href="https://github.com/easy-4-rust/sa-token-rs/tree/main/crates/sa-token-demo/sa-token-demo-axum"
	target="_blank">
	本章代码示例：Sa-Token-Rs 注解鉴权 —— [ sa-token-demo-axum ]
</a>
<a class="dt-btn" href="https://www.wenjuan.ltd/s/ARJvIbA/" target="_blank">本章小练习：Sa-Token 基础 - 注解鉴权，章节测试</a>
