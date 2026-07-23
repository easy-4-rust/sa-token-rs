# 同端互斥登录

> Sa-Token → Sa-Token-Rs。关键配置：`is_concurrent = false`，登录时指定 `device_type`。

| Java | Rust |
|---|---|
| `sa-token.is-concurrent=false` | `is_concurrent: false` |
| `StpUtil.login(10001, "PC")` | `StpUtil::login_with_device("10001", "PC")?` |
| `StpUtil.logout(10001, "PC")` | 见下文注销说明 |
| `StpUtil.getLoginDevice()` | `StpUtil::get_login_device_type()?` |
| `StpUtil.getTokenValueByLoginId(10001, "APP")` | 见下文 Id 反查说明 |
| `NotLoginException` 场景值 `-4` / `-2` | `SaTokenException::NotLogin`（scene 字段） |

如果你经常使用腾讯QQ，就会发现它的登录有如下特点：它可以手机电脑同时在线，但是不能在两个手机上同时登录一个账号。 <br/>
同端互斥登录，指的就是：像腾讯QQ一样，在同一类型设备上只允许单地点登录，在不同类型设备上允许同时在线。


<button class="show-img" img-src="/big-file/doc/up/g3--mutex-login.gif">加载动态演示图</button>

--- 

## 具体API

在 Sa-Token-Rs 中如何做到同端互斥登录? <br/>
首先在配置中，将 `is_concurrent` 配置为 `false`，然后调用登录等相关接口时声明设备类型即可：

``` rust
use std::sync::Arc;
use sa_token::prelude::*;

SaManager::set_config(Arc::new(SaTokenConfig {
    is_concurrent: false, // 不允许同一账号同端多地同时登录
    ..Default::default()
}));
```


#### 指定设备类型登录
``` rust
use sa_token::prelude::*;

// 指定`账号id`和`设备类型`进行登录
StpUtil::login_with_device("10001", "PC")?;
```
调用此方法登录后，同设备的会被顶下线（不同设备不受影响），再次访问系统时会抛出 `SaTokenException::NotLogin` 异常（对应 Java 场景值=`-4`，顶人下线）。


#### 指定设备类型强制注销
``` rust
use sa_token::prelude::*;

// 按 loginId 强制注销该账号全部在线端
StpUtil::logout_by_login_id("10001")?;

// 若只需注销某一设备：遍历终端列表，按 device_type 过滤后 logout_by_token_value
let terminals = StpUtil::get_terminal_list_by_login_id("10001")?;
for ter in terminals {
    if ter.device_type() == "PC" {
        StpUtil::logout_by_token_value(ter.token_value())?;
    }
}
```
Java 的 `StpUtil.logout(10001, "PC")` 第二参数为设备类型；Rust 门面当前以 `logout_by_login_id`（全端）+ 按终端过滤为主。被踢出者再次访问系统时会抛出 `NotLogin` 异常（对应场景值=`-2`）。


#### 查询当前登录的设备类型
``` rust
use sa_token::prelude::*;

// 返回当前 token 的登录设备类型
StpUtil::get_login_device_type()?;
```


#### Id 反查 Token
``` rust
use sa_token::prelude::*;

// 获取指定 loginId 的 tokenValue（默认取一端）
StpUtil::get_token_value_by_login_id("10001")?;

// 指定设备类型端：从终端列表中过滤
let terminals = StpUtil::get_terminal_list_by_login_id("10001")?;
let app_token = terminals
    .iter()
    .find(|t| t.device_type() == "APP")
    .map(|t| t.token_value().to_string());
```


--- 

<a class="case-btn" href="https://github.com/sa-token-rust/sa-token-rs/tree/main/crates/sa-token-demo"
	target="_blank">
	本章代码示例：Sa-Token-Rs 同端互斥登录  —— [ login_with_device ]
</a>
