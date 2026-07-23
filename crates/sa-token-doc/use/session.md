# Session 会话

> **⚠️ 文档适配状态**：本文档为 Java 官方文档的 Rust 移植版，代码示例使用
> **axum + sa-token-rs** 栈，Session 模型与 Java 一致（Account / Token / Custom）。

--- 

### 1、Session 是什么？

Session 是会话中专业的数据缓存组件，通过 Session 我们可以很方便的缓存一些高频读写数据，提高程序性能，例如：

``` rust
use sa_token_core::StpUtil;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
struct SysUser {
    id: i64,
    name: String,
}

// 在登录时缓存 user 对象
StpUtil::get_session()
    .await?
    .set("user", &SysUser { id: 10001, name: "zhang".into() })
    .await?;

// 然后我们就可以在任意处使用这个 user 对象
let user: Option<SysUser> = StpUtil::get_session().await?.get("user").await?;
```

在 Sa-Token-Rs 中，Session 分为三种，分别是：

- `Account-Session`：指的是框架为每个 账号id 分配的 Session 
- `Token-Session`：指的是框架为每个 token 分配的 Session  
- `Custom-Session`：指的是以一个 特定的值 作为 SessionId，来分配的 Session 

> [!TIP| style:callout]
> 有关 Account-Session 与 Token-Session 的详细区别，可参考：[Session 模型详解](/fun/session-model)


### 2、Account-Session

有关 账号-Session 的 API 如下：

``` rust
// 获取当前账号 id 的 Account-Session (必须是登录后才能调用)
let session: SaSession = StpUtil::get_session().await?;

// 获取当前账号 id 的 Account-Session, 并决定在 Session 尚未创建时，是否新建并返回
let session: SaSession = StpUtil::get_session_or_create(true).await?;

// 获取账号 id 为 10001 的 Account-Session
let session: SaSession = StpUtil::get_session_by_login_id(10001_i64).await?;

// 获取账号 id 为 10001 的 Account-Session, 并决定在 Session 尚未创建时，是否新建并返回
let session: SaSession = StpUtil::get_session_by_login_id_or(10001_i64, true).await?;

// 获取 SessionId 为 xxxx-xxxx 的 Account-Session, 在 Session 尚未创建时, 返回 `None`
let session: Option<SaSession> = StpUtil::get_session_by_id("xxxx-xxxx").await?;
```


### 3、Token-Session

有关 令牌-Session 的 API 如下：

``` rust
// 获取当前 Token 的 Token-Session 对象
let session: SaSession = StpUtil::get_token_session().await?;

// 获取指定 Token 的 Token-Session 对象
let session: SaSession = StpUtil::get_token_session_by_token("token_value").await?;
```


### 4、Custom-Session

自定义 Session 指的是以一个`特定的值`作为 SessionId 来分配的`Session`，借助自定义 Session，你可以为系统中的任意元素分配相应的 session。<br>
例如以商品 id 作为 key 为每个商品分配一个 Session，以便于缓存和商品相关的数据，其相关 API 如下：

``` rust
use sa_token_core::SaSessionCustomUtil;

// 查询指定 key 的 Session 是否存在
let exists: bool = SaSessionCustomUtil::is_exists("goods-10001").await?;

// 获取指定 key 的 Session，如果没有，则新建并返回
let session: SaSession = SaSessionCustomUtil::get_session_by_id("goods-10001").await?;

// 获取指定 key 的 Session，如果没有，第二个参数决定是否新建并返回  
let session: SaSession = SaSessionCustomUtil::get_session_by_id_or("goods-10001", false).await?;   

// 删除指定 key 的 Session
SaSessionCustomUtil::delete_session_by_id("goods-10001").await?;
```


### 5、在 Session 上存取值

以上三种 Session 均为框架设计概念上的区分，实际上在获取它们时，返回的都是 `SaSession` 对象，你可以使用以下 API 在 `SaSession` 对象上存取值：

``` rust
use sa_token_core::SaSession;

// 写值 
session.set("name", "zhang").await?; 

// 写值 (只有在此 key 原本无值的时候才会写入)
session.set_default_value("name", "zhang").await?;

// 取值
let v: Option<String> = session.get::<String>("name").await?;

// 取值 (指定默认值)
let v: String = session.get_or("name", "<defaultValue>").await?; 

// 取值 (若无值则执行闭包，将结果保存到此键名下并返回；
//        若有值则直接返回，不执行闭包)
let v: String = session.get_or_else("name", || async {
    // ... 计算默认值
    "computed".to_string()
}).await?;

// ---------- 数据类型转换： ----------
let age: i32     = session.get_int("age").await?;     // 转 i32 类型
let age: i64     = session.get_long("age").await?;    // 转 i64 类型
let name: String = session.get_string("name").await?; // 转 String 类型
let r: f64       = session.get_double("result").await?; // 转 f64 类型
let r: f32       = session.get_float("result").await?;  // 转 f32 类型
let m: Student   = session.get_model::<Student>("key").await?;     // 指定转换类型
let m: Student   = session.get_model_or::<Student>("key", default).await?; // 指定转换类型 + 默认值

// 是否含有某个 key (返回 true 或 false)
let b: bool = session.has("key").await?; 

// 删值 
session.delete("name").await?;          

// 清空所有值 
session.clear().await?;                 

// 获取此 Session 的所有 key (返回 `Vec<String>`)
let keys: Vec<String> = session.keys().await?;      
```


### 6、其它操作

``` rust
// 返回此 Session 的 id 
let id: String = session.get_id();                          

// 返回此 Session 的创建时间 (时间戳，单位 s) 
let created_at: i64 = session.get_create_time().await?;                  

// 返回此 Session 会话上的底层数据对象（如果更新 map 里的值，
// 请调用 session.update() 方法避免产生脏数据）
let data_map: &Map<String, Value> = session.get_data_map();                     

// 将这个 Session 从持久库更新一下
session.update().await?;                         

// 注销此 Session 会话 (从持久库删除此 Session)
session.logout().await?;                         
```


### 7、避免与 axum 状态/HttpSession 混淆使用

经常有同学会把 `SaSession` 与 Web 框架自带的会话抽象进行混淆，例如：

``` rust
use axum::extract::State;
use axum::http::StatusCode;

// 把用户数据写入 axum 的 State （不要这样做！）
async fn reset(State(state): State<MyState>) -> StatusCode {
    state.user_count = 66;
    // 在 SaSession 进行取值
    let name: Option<String> = StpUtil::get_session().await
        .ok()
        .and_then(|s| futures::executor::block_on(s.get::<String>("name")))
        .flatten();
    assert!(name.is_none()); // 输出 None
    StatusCode::OK
}
```

**要点：**

1. `SaSession` 与 Web 框架自带的会话状态没有任何关系，在 Web 框架状态上写入的值，在 `SaSession` 中无法取出。
2. Web 框架的会话能力并未被框架接管，在使用 Sa-Token-Rs 时，请在任何情况下均使用 `SaSession`，不要使用框架原生的会话抽象。


### 8、未登录场景下获取 Token-Session 

默认场景下，只有登录后才能通过 `StpUtil::get_token_session()` 获取 `Token-Session`。

如果想要在未登录场景下获取 Token-Session，有两种方法：

- 方法一：将全局配置项 `token_session_check_login` 改为 `false`，详见：[框架配置](/use/config?id=所有可配置项)
- 方法二：使用匿名 Token-Session

``` rust
// 获取当前 Token 的匿名 Token-Session （可在未登录情况下使用的 Token-Session）
let anon: SaSession = StpUtil::get_anon_token_session().await?;
```

注意点：如果前端没有提交 Token，或者提交的 Token 是一个无效 Token 的话，框架将不会根据此 Token 创建 `Token-Session` 对象，
而是随机一个新的 Token 值来创建 `Token-Session` 对象，此 Token 值可以通过 `StpUtil::get_token_value()` 获取到。


---

<a class="case-btn" href="https://github.com/your-org/sa-token-rs/blob/main/crates/sa-token-demo-axum/src/session_controller.rs"
	target="_blank">
	本章代码示例：Sa-Token-Rs Session 会话 —— [ session_controller.rs ]
</a>
<a class="dt-btn" href="https://www.wenjuan.ltd/s/MNnUr2V/" target="_blank">本章小练习：Sa-Token 基础 - Session 会话，章节测试</a>