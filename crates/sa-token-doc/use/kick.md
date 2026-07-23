# 踢人下线

所谓踢人下线，核心操作就是找到指定 `loginId` 对应的 `Token`，并设置其失效。

<img src="/big-file/doc/use/kickout.png" alt="踢下线">

> | Java | Rust |
> |---|---|
> | `StpUtil.logout(10001)` | `StpUtil::logout_by_login_id("10001")?` |
> | `StpUtil.kickout(10001)` | `StpUtil::kickout("10001")?` |
> | `StpUtil.replaced(10001)` | `StpUtil::replaced("10001")?` |

---


### 1、强制注销

``` rust
StpUtil::logout_by_login_id("10001")?;           // 强制指定账号注销下线
// 指定端注销：结合登录参数 / 终端列表 API，见 [登录参数](/up/login-parameter)
StpUtil::logout_by_token_value("token")?;        // 强制指定 Token 注销下线
```

Java 对照：

``` java
StpUtil.logout(10001);
StpUtil.logout(10001, "PC");
StpUtil.logoutByTokenValue("token");
```


### 2、踢人下线

``` rust
StpUtil::kickout("10001")?;                      // 将指定账号踢下线
// 指定端踢下线：见登录参数 / 终端 API
StpUtil::kickout_by_token_value("token")?;       // 将指定 Token 踢下线
```

强制注销 和 踢人下线 的区别在于：
- 强制注销等价于对方主动调用了注销方法，再次访问会提示：Token无效。
- 踢人下线不会清除Token信息，而是将其打上特定标记，再次访问会提示：Token已被踢下线。


<button class="show-img" img-src="/big-file/doc/use/g3--kickout.gif">加载动态演示图</button>


### 3、顶人下线

“顶人下线” 操作发生在框架登录时顶退旧登录设备，属于框架内部操作，一般情形下你不会调用到此 API：

``` rust
StpUtil::replaced("10001")?;                     // 将指定账号顶下线
StpUtil::replaced_by_token_value("token")?;      // 将指定 Token 顶下线
```


---

<a class="case-btn" href="https://github.com/easy-4-rust/sa-token-rs/tree/main/crates/sa-token-demo/sa-token-demo-axum"
	target="_blank">
	本章代码示例：Sa-Token-Rs 踢人下线 —— [ sa-token-demo-axum ]
</a>
<a class="dt-btn" href="https://www.wenjuan.ltd/s/MFNN7bK/" target="_blank">本章小练习：Sa-Token 基础 - 踢人下线，章节测试</a>
