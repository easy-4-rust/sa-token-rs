# Session会话

> 保留 Java 原文章节结构；API 改为 Sa-Token-Rs 同步门面（无 `.await`）。

| Java | Rust |
|---|---|
| `StpUtil.getSession().set("k", v)` | `StpUtil::get_session()?.set("k", json!(v))` |
| `StpUtil.getSession().get("k")` | `StpUtil::get_session()?.get("k")` |
| `StpUtil.getTokenSession()` | `StpUtil::get_token_session()?` |
| `StpUtil.getAnonTokenSession()` | `StpUtil::get_anon_token_session()?` |

--- 

### 1、Session是什么？

Session 是会话中专业的数据缓存组件，通过 Session 我们可以很方便的缓存一些高频读写数据，提高程序性能，例如：

``` rust
use sa_token::prelude::*;
use serde_json::json;

// 在登录时缓存 user 对象
let mut session = StpUtil::get_session()?;
session.set("user", json!({ "id": 10001, "name": "zhang" }));

// 然后我们就可以在任意处使用这个 user 对象
let user = StpUtil::get_session()?.get("user").cloned();
```

在 Sa-Token-Rs 中，Session 分为三种，分别是：

- `Account-Session`: 指的是框架为每个 账号id 分配的 Session 
- `Token-Session`: 指的是框架为每个 token 分配的 Session  
- `Custom-Session`: 指的是以一个 特定的值 作为SessionId，来分配的 Session 

> [!TIP| style:callout] 
> 有关 Account-Session 与 Token-Session 的详细区别，可参考：[Session模型详解](/fun/session-model)


### 2、Account-Session

有关 账号-Session 的 API 如下：

``` rust
// 获取当前账号 id 的 Account-Session (必须是登录后才能调用)
StpUtil::get_session()?;

// 获取账号 id 为 10001 的 Account-Session
StpUtil::get_session_by_login_id("10001")?;
```

Java 中 `getSession(true)` / `getSessionByLoginId(id, true)` 的「是否新建」语义，以当前 Rust API / DAO 行为为准；若无对应重载，请先登录或确保 Session 已创建。


### 3、Token-Session

有关 令牌-Session 的 API 如下：

``` rust
// 获取当前 Token 的 Token-Session 对象
StpUtil::get_token_session()?;

// 获取指定 Token 的 Token-Session 对象
StpUtil::get_token_session_by_token(&token)?;
```


### 4、Custom-Session

自定义 Session 指的是以一个`特定的值`作为 SessionId 来分配的`Session`, 借助自定义Session，你可以为系统中的任意元素分配相应的session<br>
例如以商品 id 作为 key 为每个商品分配一个Session，以便于缓存和商品相关的数据。

Java 中的 `SaSessionCustomUtil` 在 Rust 侧请以 `sa-token-core` 是否提供等价工具为准；若暂无独立工具类，可通过 DAO / Session 工厂自行以自定义 id 读写。

``` rust
// 示意：以业务 key 作为 session id 读写（具体 API 以实现为准）
// SaSessionCustomUtil::get_session_by_id("goods-10001")
```


### 5、在 Session 上存取值

以上三种 Session 均为框架设计概念上的区分，实际上在获取它们时，返回的都是 `SaSession` 对象，你可以使用以下 API 在 `SaSession` 对象上存取值：

``` rust
use serde_json::json;

// 写值
session.set("name", json!("zhang"));

// 取值
session.get("name");

// ---------- 数据类型转换（结合 serde_json::Value） ----------
// session.get("age").and_then(|v| v.as_i64());
// session.get("name").and_then(|v| v.as_str());
// 或反序列化到结构体：serde_json::from_value::<Student>(...)

// 是否含有某个 key
session.get("key").is_some();

// 删值 / 清空 —— 以 SaSession 实际方法为准（delete / clear / keys）
```


### 6、其它操作

``` rust
// 返回此 Session 的 id、创建时间、底层数据等 —— 以 SaSession 字段 / 方法为准
// session.id()
// session.create_time()
// 持久化更新 / 注销：session.update() / session.logout()（若已提供）
```


### 7、避免与 Web 框架自带 Session 混淆使用

经常有同学会把 `SaSession` 与 Web 框架自带的 Session（如 axum 社区 session 中间件、actix session）进行混淆，例如：在框架 Session 写入的值，在 `SaSession` 中无法取出。

**要点：**
1. `SaSession` 与 Web 框架 Session 没有任何关系，在框架 Session 上写入的值，在 `SaSession` 中无法取出。
2. 在使用 Sa-Token-Rs 时，请在任何情况下均使用 `SaSession`，不要与其它 Session 混用。


### 8、未登录场景下获取 Token-Session 

默认场景下，只有登录后才能通过 `StpUtil::get_token_session()` 获取 `Token-Session`。

如果想要在未登录场景下获取 Token-Session ，有两种方法：

- 方法一：将全局配置项 `token_session_check_login`（若存在）改为 false，详见：[框架配置](/use/config)
- 方法二：使用匿名 Token-Session

``` rust
// 获取当前 Token 的匿名 Token-Session （可在未登录情况下使用的 Token-Session）
StpUtil::get_anon_token_session()?;
```

注意点：如果前端没有提交 Token ，或者提交的 Token 是一个无效 Token 的话，框架将不会根据此 Token 创建 `Token-Session` 对象，
而是随机一个新的 Token 值来创建 `Token-Session` 对象，此 Token 值可以通过 `StpUtil::get_token_value()` 获取到。


---

<a class="case-btn" href="https://github.com/easy-4-rust/sa-token-rs/tree/main/crates/sa-token-demo/sa-token-demo-axum"
	target="_blank">
	本章代码示例：Sa-Token-Rs Session 会话 —— [ sa-token-demo-axum ]
</a>
<a class="dt-btn" href="https://www.wenjuan.ltd/s/MNnUr2V/" target="_blank">本章小练习：Sa-Token 基础 - Session 会话，章节测试</a>
