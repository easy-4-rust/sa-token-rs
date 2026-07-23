# 登录参数

> 保留原文章节；API 改为 Sa-Token-Rs。`SaLoginParameter` 字段为 snake_case，链式 `set_*` 返回 `Self`。

| Java | Rust |
|---|---|
| `StpUtil.login(10001, "PC")` | `StpUtil::login_with_device("10001", "PC")?` |
| `StpUtil.login(id, new SaLoginParameter()...)` | `StpUtil::login_with_param(id, &param)?` |
| `StpUtil.logoutByTokenValue` | `StpUtil::logout_by_token_value` |

### 1、登录参数

在之前的章节我们提到，通过 `StpUtil::login(xxx)` 可以完成指定账号登录，同时你可以指定设备类型等扩展登录信息，比如：

``` rust
// 指定`账号id`和`设备类型`进行登录
StpUtil::login_with_device("10001", "PC")?;

// 「记住我」等语义可通过 SaLoginParameter / Cookie 配置表达
```

除了以上内容，你还可以指定一个 `SaLoginParameter` 对象，来详细控制登录的多个细节，例如：

``` rust
use sa_token::prelude::*;
use serde_json::json;

let param = SaLoginParameter::create()
    .set_device_type("PC")              // 此次登录的客户端设备类型, 一般用于完成 [同端互斥登录] 功能
    .set_device_id("xxxxxxxxx")         // 此次登录的客户端设备ID, 登录成功后该设备将标记为可信任设备
    .set_is_lasting_cookie(true)        // 是否为持久Cookie
    .set_timeout(60 * 60 * 24 * 7)      // 指定此次登录 token 的有效期, 单位:秒，-1=永久有效
    .set_active_timeout(60 * 60 * 24 * 7) // 指定此次登录 token 的最低活跃频率, 单位:秒
    .set_is_concurrent(true)            // 是否允许同一账号多地同时登录
    .set_is_share(false)                // 多人登录同一账号时是否共用一个 token
    .set_max_login_count(12)            // 同一账号最大登录数量，-1代表不限
    .set_extra_data(json!({"key": "value"})) // Token 扩展参数（jwt 等场景）
    .set_token("xxxx-xxxx-xxxx-xxxx")   // 预定此次登录的生成的Token
    .set_is_write_header(false)         // 是否在登录后将 Token 写入到响应头
    .set_terminal_extra_data(json!({"key": "value"})); // 挂载到 SaTerminalInfo 的扩展数据

StpUtil::login_with_param("10001", &param)?;
```

以上大部分参数在未指定时将使用全局配置作为默认值。

Java 原文中的 `setReplacedRange` / `setOverflowLogoutMode` / `setupCookieConfig` 等，若当前 `SaLoginParameter` 尚未全部暴露，请以全局 `SaTokenConfig` 与源码为准；概念说明仍见 [框架配置](/use/config)。



### 2、注销参数

同样的，在调用注销时，也可以指定一些参数决定注销的细节行为：

``` rust
use sa_token::prelude::*;

// 当前客户端注销
StpUtil::logout()?;

// 指定 token 注销
StpUtil::logout_by_token_value("xxxxxxxxxxxxxxxxxxxxxxx")?;

// 指定 loginId 注销
StpUtil::logout_by_login_id("10001")?;

// 踢人 / 顶人
StpUtil::kickout("10001")?;
StpUtil::replaced("10001")?;

// 注销参数对象（对应 Java SaLogoutParameter）
let _logout_param = SaLogoutParameter::create()
    .set_device_type("PC")
    .set_mode(SaLogoutMode::Logout)
    .set_range(SaLogoutRange::Token)
    .set_is_keep_freeze_ops(false)
    .set_is_keep_token_session(false);
// 若门面尚未暴露 logout_with_param，可结合全局 SaTokenConfig.logout_range
// 与上述 logout_* / kickout / replaced API 组合使用。
```

以上大部分参数在未指定时将使用全局配置作为默认值。


### 3、遍历登录终端详细操作

如果你的 登录策略 或 注销策略 非常复杂，凭借上述参数无法组合出你的业务场景，你可以手动遍历一个账号的已登录终端信息列表，手动决定某个设备是否下线，例如：

``` rust
use axum::Json;
use sa_token::prelude::*;
use serde_json::{json, Value};

/// 遍历账号已登录终端列表，进行详细操作
async fn logout_selective() -> SaResult<Json<Value>> {
    let terminals = StpUtil::get_terminal_list_by_login_id("10001")?;
    for ter in terminals {
        // 根据登录顺序，奇数的保留，偶数的下线
        if ter.index % 2 == 0 {
            StpUtil::logout_by_token_value(&ter.token_value)?; // 注销下线
            // StpUtil::kickout_by_token_value(&ter.token_value)?; // 踢人下线
            // StpUtil::replaced_by_token_value(&ter.token_value)?; // 顶人下线
        }
    }
    Ok(Json(json!({ "code": 200 })))
}
```

Java 中的 `forEachTerminalList` / `removeTerminalByLogout` 等，Rust 侧用 `get_terminal_list_by_login_id` + `logout_by_token_value` 等组合表达。
