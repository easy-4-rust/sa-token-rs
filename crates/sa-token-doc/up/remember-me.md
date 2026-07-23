# [记住我] 模式
--- 

> Sa-Token → Sa-Token-Rs。记住我语义由 `is_lasting_cookie` 控制（全局 `SaTokenConfig` 或单次 `SaLoginParameter`）。

| Java | Rust |
|---|---|
| `StpUtil.login(10001, false)` | `StpUtil::login_with_param("10001", &SaLoginParameter::create().set_is_lasting_cookie(false))?` |
| `SaLoginParameter.setIsLastingCookie(true)` | `.set_is_lasting_cookie(true)` |
| `SaLoginParameter.setTimeout(...)` | `.set_timeout(...)` |
| `SaLoginParameter.setDevice("PC")` | `.set_device_type("PC")` |

如图所示，一般网站的登录界面都会有一个 **`[记住我]`** 按钮，当你勾选它登录后，即使你关闭浏览器再次打开网站，也依然会处于登录状态，无须重复验证密码：

<img src="/big-file/doc/up/login-view.png" alt="../static/login-view.png">

那么在Sa-Token-Rs中，如何做到 [ 记住我 ] 功能呢？


### 在 Sa-Token-Rs 中实现记住我功能

Sa-Token-Rs的登录授权，**默认就是`[记住我]`模式**（`SaTokenConfig.is_lasting_cookie` 默认为 `true`），为了实现`[非记住我]`模式，你需要在登录时如下设置：

``` rust
use sa_token::prelude::*;

// 设置登录账号 id 为 10001，is_lasting_cookie=false 表示非记住我：
// 关闭浏览器后再次打开需要重新登录
let param = SaLoginParameter::create().set_is_lasting_cookie(false);
StpUtil::login_with_param("10001", &param)?;
```

那么，Sa-Token-Rs实现`[记住我]`的具体原理是？


### 实现原理
Cookie作为浏览器提供的默认会话跟踪机制，其生命周期有两种形式，分别是：
- 临时Cookie：有效期为本次会话，只要关闭浏览器窗口，Cookie就会消失。
- 持久Cookie：有效期为一个具体的时间，在时间未到期之前，即使用户关闭了浏览器Cookie也不会消失。

利用Cookie的此特性，我们便可以轻松实现 [记住我] 模式：
- 勾选 [记住我] 按钮时：调用 `login_with_param(..., set_is_lasting_cookie(true))`，在浏览器写入一个`持久Cookie`储存 Token，此时用户即使重启浏览器 Token 依然有效。
- 不勾选 [记住我] 按钮时：调用 `set_is_lasting_cookie(false)`，在浏览器写入一个`临时Cookie`储存 Token，此时用户在重启浏览器后 Token 便会消失，导致会话失效。


<button class="show-img" img-src="/big-file/doc/up/g3--remember-me.gif">加载动态演示图</button>


### 前后端分离模式下如何实现[记住我]?

此时机智的你😏很快发现一个问题，Cookie虽好，却无法在前后端分离环境下使用，那是不是代表上述方案在APP、小程序等环境中无效？

准确的讲，答案是肯定的，任何基于Cookie的认证方案在前后端分离环境下都会失效（原因在于这些客户端默认没有实现Cookie功能），不过好在，这些客户端一般都提供了替代方案，
唯一遗憾的是，此场景中token的生命周期需要我们在前端手动控制：

以经典跨端框架 [uni-app](https://uniapp.dcloud.io/) 为例，我们可以使用如下方式达到同样的效果：
``` js
// 使用本地存储保存token，达到 [持久Cookie] 的效果
uni.setStorageSync("satoken", "xxxx-xxxx-xxxx-xxxx-xxx");

// 使用globalData保存token，达到 [临时Cookie] 的效果
getApp().globalData.satoken = "xxxx-xxxx-xxxx-xxxx-xxx";
```

如果你决定在PC浏览器环境下进行前后端分离模式开发，那么更加简单：
``` js
// 使用 localStorage 保存token，达到 [持久Cookie] 的效果
localStorage.setItem("satoken", "xxxx-xxxx-xxxx-xxxx-xxx");

// 使用 sessionStorage 保存token，达到 [临时Cookie] 的效果
sessionStorage.setItem("satoken", "xxxx-xxxx-xxxx-xxxx-xxx");
```

Remember me, it's too easy!



### 登录时指定 Token 有效期
登录时不仅可以指定是否为`[记住我]`模式，还可以指定一个特定的时间作为 Token 有效时长，如下示例：
``` rust
use sa_token::prelude::*;

// 示例1：
// 指定token有效期(单位: 秒)，如下所示token七天有效
StpUtil::login_with_param(
    "10001",
    &SaLoginParameter::create().set_timeout(60 * 60 * 24 * 7),
)?;

// ----------------------- 示例2：所有参数
// `SaLoginParameter`为登录参数Model，其有诸多参数决定登录时的各种逻辑，例如：
let param = SaLoginParameter::create()
    .set_device_type("PC")                 // 此次登录的客户端设备类型, 用于[同端互斥登录]
    .set_is_lasting_cookie(true)           // 是否为持久Cookie
    .set_timeout(60 * 60 * 24 * 7)         // 指定此次登录token的有效期, 单位:秒
    .set_token("xxxx-xxxx-xxxx-xxxx")      // 预定此次登录的生成的Token
    .set_is_write_header(false);           // 是否在登录后将 Token 写入到响应头

StpUtil::login_with_param("10001", &param)?;
```

参考 demo：`sa-token-demo-axum-remember-me` / `sa-token-demo-actix-remember-me`。



--- 

<a class="case-btn" href="https://github.com/sa-token-rust/sa-token-rs/tree/main/crates/sa-token-demo"
	target="_blank">
	本章代码示例：Sa-Token-Rs 记住我登录 —— [ remember-me demo ]
</a>
