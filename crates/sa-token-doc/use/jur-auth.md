# 权限认证

> 保留 Java 原文章节结构；示例改为 Sa-Token-Rs。

| Java | Rust |
|---|---|
| `implements StpInterface` | `impl StpInterface for ...` |
| `@Component` 扫描注册 | `SaManager::set_stp_interface(Arc::new(...))` |
| `StpUtil.checkPermission("x")` | `StpUtil::check_permission("x")?` |
| `NotPermissionException` | `SaTokenException::NotPermission`（或等价 Err） |

--- 

### 1、设计思路

权限认证的最终目的在于：规定哪些用户可以访问哪些 接口/页面/资源。

例如对于同一个页面：
- 管理员账号访问：<green>正常返回数据</green>。
- 普通账号访问：<red>权限不足，拒绝访问</red>。

<img class="w-100" src="/big-file/doc/use/use-jur-auth.svg" />


那么框架是如何判断，一个账号是否有权限访问某个接口的呢？

从底层数据的角度来讲，<green>**每个账号都会拥有一组权限码集合，框架要做的就是校验这个集合中是否包含指定的权限码。**</green>

- 有，就让你通过。
- 没有？那么禁止访问！

<img class="w-100" src="/big-file/doc/use/use-jur-check.svg" />


所以现在问题的核心就是两个：
1. 如何定义一个账号所拥有的权限码集合？
2. 本次操作需要校验的权限码是哪个？


### 2、获取当前账号权限码集合

在进行具体的权限校验之前，你需要实现 `StpInterface` trait，告诉框架指定账号拥有的权限码集合是哪些：

``` rust
use sa_token::prelude::*;
use sa_token::sa_token_core::stp::StpInterface;
use std::sync::Arc;

/// 自定义权限加载接口实现类（对应 Java StpInterfaceImpl）
pub struct StpInterfaceImpl;

impl StpInterface for StpInterfaceImpl {
    /// 返回一个账号所拥有的权限码集合
    fn get_permission_list(&self, _login_id: &str, _login_type: &str) -> Vec<String> {
        // 本 list 仅做模拟，实际项目中要根据具体业务逻辑来查询权限
        vec![
            "101".into(),
            "user.add".into(),
            "user.update".into(),
            "user.get".into(),
            // "user.delete".into(),
            "art.*".into(),
        ]
    }

    /// 返回一个账号所拥有的角色标识集合 (权限与角色可分开校验)
    fn get_role_list(&self, _login_id: &str, _login_type: &str) -> Vec<String> {
        // 本 list 仅做模拟，实际项目中要根据具体业务逻辑来查询角色
        vec!["admin".into(), "super-admin".into()]
    }
}

/// 注册到 SaManager（对应 Java @Component 自动扫描）
fn register_stp_interface() {
    SaManager::set_stp_interface(Arc::new(StpInterfaceImpl) as Arc<dyn StpInterface>);
}
```

**参数解释：**
- login_id：账号 id，即你在调用 `StpUtil::login(id)` 时写入的`唯一标识`值。
- login_type：账号体系标识，此处可以暂时忽略，在 [ 多账户认证 ] 章节下会对这个概念做详细的解释。

可参考代码：`crates/sa-token-demo/sa-token-demo-axum/src/satoken/stp_interface_impl.rs`

> [!WARNING| label:有同学会产生疑问：我实现了此接口，但是程序启动时好像并没有执行，是不是我写错了？]
> 答：不执行是正常现象，程序启动时不会执行这个接口的方法，在每次调用鉴权代码时，才会执行到此。


### 3、权限校验

然后就可以用以下 api 来鉴权了

``` rust
// 获取：当前账号所拥有的权限集合
StpUtil::get_permission_list()?;

// 判断：当前账号是否含有指定权限, 返回 Ok(true) 或 Ok(false)
StpUtil::has_permission("user.add")?;

// 校验：当前账号是否含有指定权限, 如果验证未通过，则返回 Err（对应 NotPermissionException）
StpUtil::check_permission("user.add")?;

// 校验：当前账号是否含有指定权限 [指定多个，必须全部验证通过]
StpUtil::check_permission_and(&["user.add", "user.delete", "user.get"])?;

// 校验：当前账号是否含有指定权限 [指定多个，只要其一验证通过即可]
StpUtil::check_permission_or(&["user.add", "user.delete", "user.get"])?;
```

扩展：错误信息中通常会带有 login_type / 权限码等上下文，便于排查是哪个 `StpLogic` 抛出的。


### 4、角色校验

在 Sa-Token-Rs 中，角色和权限可以分开独立验证

``` rust
// 获取：当前账号所拥有的角色集合
StpUtil::get_role_list()?;

// 判断：当前账号是否拥有指定角色
StpUtil::has_role("super-admin")?;

// 校验：当前账号是否含有指定角色标识, 如果验证未通过，则返回 Err（对应 NotRoleException）
StpUtil::check_role("super-admin")?;

// 校验：当前账号是否含有指定角色标识 [指定多个，必须全部验证通过]
StpUtil::check_role_and(&["super-admin", "shop-admin"])?;

// 校验：当前账号是否含有指定角色标识 [指定多个，只要其一验证通过即可]
StpUtil::check_role_or(&["super-admin", "shop-admin"])?;
```


### 5、拦截全局异常

有同学要问，鉴权失败，抛出异常，然后呢？要把异常显示给用户看吗？**当然不可以！**

你可以创建一个全局错误处理，统一返回给前端的格式，参考：

``` rust
use axum::{http::StatusCode, response::IntoResponse, Json};
use serde_json::json;

/// 对应 Java @RestControllerAdvice + @ExceptionHandler
fn handler_exception(msg: impl ToString) -> impl IntoResponse {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(json!({ "code": 500, "msg": msg.to_string() })),
    )
}
```

可参考：`sa-token-demo-axum` / `sa-token-demo-axum-case` 中的错误映射写法。


### 6、权限通配符

Sa-Token-Rs 允许你根据通配符指定**泛权限**，例如当一个账号拥有`art.*`的权限时，`art.add`、`art.delete`、`art.update`都将匹配通过（以实现为准；若当前版本为精确匹配，可在 `StpInterface` 中自行展开）。

``` rust
// 当拥有 art.* 权限时
StpUtil::has_permission("art.add")?;        // true
StpUtil::has_permission("art.update")?;     // true
StpUtil::has_permission("goods.add")?;      // false

// 当拥有 *.delete 权限时
StpUtil::has_permission("art.delete")?;      // true
StpUtil::has_permission("user.delete")?;     // true
StpUtil::has_permission("user.update")?;     // false

// 当拥有 *.js 权限时
StpUtil::has_permission("index.js")?;        // true
StpUtil::has_permission("index.css")?;       // false
StpUtil::has_permission("index.html")?;      // false
```

> [!WARNING| label:上帝权限]
> 当一个账号拥有 `"*"` 权限时，他可以验证通过任何权限码 （角色认证同理）


### 7、如何把权限精确到按钮级？

权限精确到按钮级的意思就是指：**权限范围可以控制到页面上的每一个按钮是否显示**。

<img class="w-100" src="/big-file/doc/use/use-jur-btn.svg" />

思路：如此精确的范围控制只依赖后端已经难以完成，此时需要前端进行一定的逻辑判断。

如果是前后端一体项目，可以参考：[Askama 标签方言](/plugin/thymeleaf-extend)（原 Thymeleaf）或 [Tera 集成](/plugin/freemarker-extend)（原 Freemarker）；如果是前后端分离项目，则：

1. 在登录时，把当前账号拥有的所有权限码一次性返回给前端。
2. 前端将权限码集合保存在`localStorage`或其它全局状态管理对象中。
3. 在需要权限控制的按钮上，使用 js 进行逻辑判断，例如在`Vue`框架中我们可以使用如下写法：
``` js
// `arr`是当前用户拥有的权限码数组
// `user.delete`是显示按钮需要拥有的权限码
// `删除按钮`是用户拥有权限码才可以看到的内容。
<div>
	<button v-if="arr.indexOf('user.get') > -1">查询用户</button>
	<button v-if="arr.indexOf('user.update') > -1">修改用户</button>
	<button v-if="arr.indexOf('user.delete') > -1">删除按钮</button>
</div>
```

以上写法只为提供一个参考示例，不同框架有不同写法，大家可根据项目技术栈灵活封装进行调用。


> [!ATTENTION| label:前端有了鉴权后端还需要鉴权吗？]
> **需要！** <br>
> 前端的鉴权只是一个辅助功能，对于专业人员这些限制都是可以轻松绕过的，为保证服务器安全：**无论前端是否进行了权限校验，后端接口都需要对会话请求再次进行权限校验！**



---

<a class="case-btn" href="https://github.com/easy-4-rust/sa-token-rs/tree/main/crates/sa-token-demo/sa-token-demo-axum"
	target="_blank">
	本章代码示例：Sa-Token-Rs 权限认证 —— [ sa-token-demo-axum ]
</a>
<a class="dt-btn" href="https://www.wenjuan.ltd/s/ZfIjYr9/" target="_blank">本章小练习：Sa-Token 基础 - 权限认证，章节测试</a>
