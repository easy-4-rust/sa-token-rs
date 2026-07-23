# 全局侦听器

--- 

> Sa-Token → Sa-Token-Rs。实现 `SaTokenListener` trait，通过 `SaManager::register_listener` / `register_listener` 注册。内置 `SaTokenListenerForLog`（`is_log = true` 时输出）。

| Java | Rust |
|---|---|
| `implements SaTokenListener` | `impl SaTokenListener for MyListener` |
| `@Component` 自动注册 | 启动时手动 `SaManager::register_listener(Arc::new(...))` |
| `SaTokenEventCenter.registerListener` | `SaManager::register_listener` / `register_listener` |
| `SaTokenListenerForSimple` | `SaTokenListenerForSimple`（空实现，可包一层） |
| `sa-token.is-log=true` | `SaTokenConfig.is_log = true` |

### 1、工作原理

Sa-Token-Rs 提供一种侦听器机制，通过注册侦听器，你可以订阅框架的一些关键性事件，例如：用户登录、退出、被踢下线等。 

事件触发流程大致如下：

<img class="s-w" src="/big-file/doc/up/sa-token-listener.svg" alt="sa-token-listener">

框架默认内置了侦听器 `SaTokenListenerForLog` 实现，功能是控制台 log 打印输出，你可以通过配置`is_log = true`开启。

要注册自定义的侦听器也非常简单：
1. 新建类型实现 `SaTokenListener` trait。
2. 将实现注册到全局监听器列表（`SaManager::register_listener`）。


### 2、自定义侦听器实现

##### 2.1、新建实现类：

新建`my_sa_token_listener.rs`，实现`SaTokenListener`：

``` rust
use std::sync::Arc;
use sa_token_core::listener::SaTokenListener;
use sa_token_core::stp::parameter::sa_login_parameter::SaLoginParameter;

/// 自定义侦听器的实现
pub struct MySaTokenListener;

impl SaTokenListener for MySaTokenListener {
    /// 每次登录时触发
    fn do_login(
        &self,
        _login_type: &str,
        _login_id: &str,
        _token_value: &str,
        _login_parameter: &SaLoginParameter,
    ) {
        println!("---------- 自定义侦听器实现 doLogin");
    }

    /// 每次注销时触发
    fn do_logout(&self, _login_type: &str, _login_id: &str, _token_value: &str) {
        println!("---------- 自定义侦听器实现 doLogout");
    }

    /// 每次被踢下线时触发
    fn do_kickout(&self, _login_type: &str, _login_id: &str, _token_value: &str) {
        println!("---------- 自定义侦听器实现 doKickout");
    }

    /// 每次被顶下线时触发
    fn do_replaced(&self, _login_type: &str, _login_id: &str, _token_value: &str) {
        println!("---------- 自定义侦听器实现 doReplaced");
    }

    /// 每次被封禁时触发
    fn do_disable(
        &self,
        _login_type: &str,
        _login_id: &str,
        _service: &str,
        _level: i32,
        _disable_time: i64,
    ) {
        println!("---------- 自定义侦听器实现 doDisable");
    }

    /// 每次被解封时触发
    fn do_untie_disable(&self, _login_type: &str, _login_id: &str, _service: &str) {
        println!("---------- 自定义侦听器实现 doUntieDisable");
    }

    /// 每次二级认证时触发
    fn do_open_safe(
        &self,
        _login_type: &str,
        _token_value: &str,
        _service: &str,
        _safe_time: i64,
    ) {
        println!("---------- 自定义侦听器实现 doOpenSafe");
    }

    /// 每次退出二级认证时触发
    fn do_close_safe(&self, _login_type: &str, _token_value: &str, _service: &str) {
        println!("---------- 自定义侦听器实现 doCloseSafe");
    }

    /// 每次创建Session时触发
    fn do_create_session(&self, _id: &str) {
        println!("---------- 自定义侦听器实现 doCreateSession");
    }

    /// 每次注销Session时触发
    fn do_logout_session(&self, _id: &str) {
        println!("---------- 自定义侦听器实现 doLogoutSession");
    }

    /// 每次Token续期时触发
    fn do_renew_timeout(
        &self,
        _login_type: &str,
        _login_id: &str,
        _token_value: &str,
        _timeout: i64,
    ) {
        println!("---------- 自定义侦听器实现 doRenewTimeout");
    }
}
```

##### 2.2、将侦听器注册到事件中心：

Rust 没有 Spring `@Component` 自动扫描，需要在应用启动时手动注册：

``` rust
use std::sync::Arc;
use sa_token::prelude::*;
use sa_token_core::listener::register_listener;

// 将侦听器注册到事件发布中心（两种写法等价）
SaManager::register_listener(Arc::new(MySaTokenListener));
// 或：
register_listener(Arc::new(MySaTokenListener));
```

事件中心 / 监听器列表的其它一些常用操作：

``` rust
use std::sync::Arc;
use sa_token::prelude::*;
use sa_token_core::listener::SaTokenListener;

// 获取已注册的所有侦听器
let list = SaManager::listeners().read().unwrap().clone();

// 重置侦听器集合
{
    let mut w = SaManager::listeners().write().unwrap();
    w.clear();
    w.push(Arc::new(MySaTokenListener) as Arc<dyn SaTokenListener>);
}

// 注册一个侦听器
SaManager::register_listener(Arc::new(MySaTokenListener));

// 清空所有已注册的侦听器
SaManager::listeners().write().unwrap().clear();

// 判断是否已经注册了指定侦听器（按指针 / 类型自行判断）
let has = !SaManager::listeners().read().unwrap().is_empty();
let _ = has;
```

> [!TIP| label:与 Java 差异]
> Java `SaTokenEventCenter` 提供 `removeListener(cls)` / `hasListener(cls)` 等按类型操作；Rust 侧监听器列表为 `Vec<Arc<dyn SaTokenListener>>`，按类型移除需自行 `downcast` 或持有注册句柄。发布事件的内部逻辑见 `SaTokenEventCenter`。

##### 2.3、启动测试：
在测试 handler 中添加登录测试代码：
``` rust
use axum::Json;
use sa_token::prelude::*;
use serde_json::{json, Value};

/// 测试登录接口
async fn login() -> SaResult<Json<Value>> {
    println!("登录前");
    StpUtil::login("10001")?;
    println!("登录后");
    Ok(Json(json!({ "code": 200 })))
}
```

启动项目，访问登录接口，观察控制台输出：

<img class="s-w-sh" src="/big-file/doc/up/sa-token-listener-println.png" alt="sa-token-listener-println">


### 3、其它注意点

##### 3.1、你可以通过组合 `SaTokenListenerForSimple` 思路快速实现一个侦听器：

Rust 的 `SaTokenListener` 方法都带有默认空实现，因此**不必**继承 Simple 类——只重写你关心的方法即可：

``` rust
use sa_token_core::listener::SaTokenListener;
use sa_token_core::stp::parameter::sa_login_parameter::SaLoginParameter;

pub struct MySaTokenListener;

impl SaTokenListener for MySaTokenListener {
    /// 每次登录时触发（其余事件使用默认空实现）
    fn do_login(
        &self,
        _login_type: &str,
        _login_id: &str,
        _token_value: &str,
        _login_parameter: &SaLoginParameter,
    ) {
        println!("---------- 自定义侦听器实现 doLogin");
    }
}
```

（若需标记“空壳”语义，仍可引用 `SaTokenListenerForSimple` 类型作为占位，但其本身不提供“继承重写”机制。）

##### 3.2、使用闭包风格 / 轻量结构体注册：
``` rust
use std::sync::Arc;
use sa_token::prelude::*;
use sa_token_core::listener::SaTokenListener;
use sa_token_core::stp::parameter::sa_login_parameter::SaLoginParameter;

struct OnLogin;

impl SaTokenListener for OnLogin {
    fn do_login(
        &self,
        _login_type: &str,
        _login_id: &str,
        _token_value: &str,
        _login_parameter: &SaLoginParameter,
    ) {
        println!("---------------- doLogin");
    }
}

SaManager::register_listener(Arc::new(OnLogin));
```

##### 3.3、使用 try-catch（`Result` / `catch_unwind`）包裹不安全的代码：
如果你认为你的事件处理代码是不安全的（代码可能在运行时抛出异常 / panic），则需要使用 `catch_unwind` 或内部 `Result` 处理，以防因为错误导致 Sa-Token-Rs 的整个登录流程被强制中断。

``` rust
use std::panic::{AssertUnwindSafe, catch_unwind};
use sa_token_core::listener::SaTokenListener;
use sa_token_core::stp::parameter::sa_login_parameter::SaLoginParameter;

struct SafeOnLogin;

impl SaTokenListener for SafeOnLogin {
    fn do_login(
        &self,
        _login_type: &str,
        _login_id: &str,
        _token_value: &str,
        _login_parameter: &SaLoginParameter,
    ) {
        let _ = catch_unwind(AssertUnwindSafe(|| {
            // 不安全代码需要写在保护块里
            // ......
        }));
    }
}
```

##### 3.4、疑问：一个项目可以注册多个侦听器吗？
可以，多个侦听器间彼此独立，互不影响，按照注册顺序依次接受到事件通知。


---

<a class="case-btn" href="https://github.com/sa-token-rust/sa-token-rs"
	target="_blank">
	本章代码示例：Sa-Token-Rs 自定义侦听器  —— [ SaTokenListener + register_listener ]
</a>
