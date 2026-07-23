# 二级认证

> Sa-Token → Sa-Token-Rs。`StpUtil` 为**同步**门面，勿对 `StpUtil::*` 假写 `.await`。仅当使用 Redis 等异步 DAO 时，才用 `AsyncStpUtil`。

| Java | Rust |
|---|---|
| `StpUtil.openSafe(120)` | `StpUtil::open_safe(120)?` |
| `StpUtil.isSafe()` | `StpUtil::is_safe()?` |
| `StpUtil.checkSafe()` | `StpUtil::check_safe()?` |
| `StpUtil.closeSafe()` | `StpUtil::close_safe()?` |
| `StpUtil.openSafe("client", 600)` | `StpUtil::open_safe_with_service("client", 600)?` |
| `StpUtil.isSafe("client")` | `StpUtil::stp_logic().is_safe_with_service("client")?` |
| `StpUtil.checkSafe("client")` | `StpUtil::stp_logic().check_safe_with_service("client")?` |
| `StpUtil.closeSafe("client")` | `StpUtil::stp_logic().close_safe_with_service("client")?` |
| `StpUtil.getSafeTime()` | 见下文「剩余有效时间」说明 |
| `@SaCheckSafe` | `SaCheckSafeMeta` / `#[sa_check_safe]`（见注解章） |

在某些敏感操作下，我们需要对已登录的会话进行二次验证。

比如代码托管平台的仓库删除操作，尽管我们已经登录了账号，当我们点击 **[删除]** 按钮时，还是需要再次输入一遍密码，这么做主要为了两点：

1. 保证操作者是当前账号本人。
2. 增加操作步骤，防止误删除重要数据。

这就是我们本篇要讲的 —— 二级认证，即：在已登录会话的基础上，进行再次验证，提高会话的安全性。


--- 

### 具体API

在 `Sa-Token-Rs` 中进行二级认证非常简单，只需要使用以下API：

``` rust
use sa_token::prelude::*;

// 在当前会话 开启二级认证，时间为120秒
StpUtil::open_safe(120)?;

// 获取：当前会话是否处于二级认证时间内
StpUtil::is_safe()?;

// 检查当前会话是否已通过二级认证，如未通过则抛出异常
StpUtil::check_safe()?;

// 获取当前会话的二级认证剩余有效时间 (单位: 秒)
// Java 有 StpUtil.getSafeTime()；当前 StpUtil 门面尚未封装同名方法，
 // 可通过 DAO 读取 safe key 的 TTL（值为 -2 代表尚未通过二级认证）：
// SaManager::sa_token_dao().get_timeout(&safe_key)?

// 在当前会话 结束二级认证
StpUtil::close_safe()?;
```


### 一个小示例

一个完整的二级认证业务流程，应该大致如下：

``` rust
use axum::Json;
use sa_token::prelude::*;
use serde_json::{json, Value};

/// 删除仓库（需先完成二级认证）
async fn delete_project(project_id: String) -> SaResult<Json<Value>> {
    // 第1步，先检查当前会话是否已完成二级认证
    if !StpUtil::is_safe()? {
        return Ok(Json(json!({
            "code": 500,
            "msg": "仓库删除失败，请完成二级认证后再次访问接口"
        })));
    }

    // 第2步，如果已完成二级认证，则开始执行业务逻辑
    let _ = project_id;
    // ...

    // 第3步，返回结果
    Ok(Json(json!({ "code": 200, "msg": "仓库删除成功" })))
}

/// 提供密码进行二级认证
async fn open_safe(password: String) -> SaResult<Json<Value>> {
    // 比对密码（此处只是举例，真实项目时可拿其它参数进行校验）
    if password == "123456" {
        // 比对成功，为当前会话打开二级认证，有效期为120秒
        StpUtil::open_safe(120)?;
        return Ok(Json(json!({ "code": 200, "msg": "二级认证成功" })));
    }

    // 如果密码校验失败，则二级认证也会失败
    Ok(Json(json!({ "code": 500, "msg": "二级认证失败" })))
}
```

Java 对照：`@RequestMapping` → axum handler；`SaResult` → `serde_json` + `SaResult`/`Json`。

> [!NOTE| label:调用步骤：] 
> 1. 前端调用 `deleteProject` 接口，尝试删除仓库。
> 2. 后端校验会话尚未完成二级认证，返回： `仓库删除失败，请完成二级认证后再次访问接口`。
> 3. 前端将信息提示给用户，用户输入密码，调用 `openSafe` 接口。
> 4. 后端比对用户输入的密码，完成二级认证，有效期为：120秒。
> 5. 前端在 120 秒内再次调用 `deleteProject` 接口，尝试删除仓库。
> 6. 后端校验会话已完成二级认证，返回：`仓库删除成功`。


### 指定业务标识进行二级认证

如果项目有多条业务线都需要敏感操作验证，则 `StpUtil::open_safe()` 无法提供细粒度的认证操作，
此时我们可以指定一个业务标识来分辨不同的业务线：

``` rust
use sa_token::prelude::*;

// 在当前会话 开启二级认证，业务标识为 client，时间为600秒
StpUtil::open_safe_with_service("client", 600)?;

// 获取：当前会话是否已完成指定业务的二级认证
StpUtil::stp_logic().is_safe_with_service("client")?;

// 校验：当前会话是否已完成指定业务的二级认证 ，如未认证则抛出异常
StpUtil::stp_logic().check_safe_with_service("client")?;

// 关闭指定业务标识的二级认证
StpUtil::stp_logic().close_safe_with_service("client")?;
```

> [!TIP| label:门面说明]
> `StpUtil` 已暴露 `open_safe_with_service`；按业务校验/关闭请走 `StpUtil::stp_logic().*_with_service(...)`（或自行在业务层再包一层静态方法）。

业务标识可以填写任意字符串，不同业务标识之间的认证互不影响，比如：
``` rust
// 打开了业务标识为 client 的二级认证
StpUtil::open_safe_with_service("client", 120)?;

// 判断是否处于 shop 的二级认证，会返回 false
StpUtil::stp_logic().is_safe_with_service("shop")?;  // false

// 也不会通过校验，会抛出异常（SaTokenException::NotSafe）
StpUtil::stp_logic().check_safe_with_service("shop")?;
```



### 使用注解进行二级认证
在一个方法上使用二级认证注解 / 元数据，可以在代码进入此方法之前进行一次二级认证校验。

Java 的 `@SaCheckSafe` / `@SaCheckSafe("art")` 在 Rust 侧对应 `SaCheckSafeMeta`（以及宏 `#[sa_check_safe]`，若已启用）：

``` rust
use sa_token_core::annotation::sa_check_safe::SaCheckSafeMeta;

// 默认业务标识（见 SaTokenConsts::DEFAULT_SAFE_AUTH_SERVICE）
let _meta = SaCheckSafeMeta::new();

// 指定业务类型
let _meta_art = SaCheckSafeMeta::with_value("art");
```

在 axum / actix 路由上，常见做法是在 handler 入口显式调用 `StpUtil::check_safe()?`，或挂载注解处理器中间件。

详细使用方法可参考：[注解鉴权](/use/at-check)，此处不再赘述



---

<a class="case-btn" href="https://github.com/sa-token-rust/sa-token-rs/tree/main/crates/sa-token-demo"
	target="_blank">
	本章代码示例：Sa-Token-Rs 二级认证 —— 参考 demo-axum `/at/openSafe`、`/at/checkSafe`
</a>
