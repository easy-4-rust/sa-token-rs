# Sa-Token-Rs 集成 Redis
---

> Sa-Token → Sa-Token-Rs。Java `RedisTemplate` 集成 → Rust `sa-token-dao-redis` + `SaTokenDaoRedis`。Redis DAO 为异步实现，推荐配合 `AsyncStpUtil` / `AsyncSaTokenRuntime`（见 demo）。

| Java | Rust |
|---|---|
| `sa-token-redis-template` | `sa-token-dao-redis` |
| `RedisTemplate` + Spring | `redis` crate + `SaTokenDaoRedis` |
| Jackson / Fastjson | `serde` / `serde_json`（DAO 内默认 JSON） |
| `application.yml` redis | 环境变量 `REDIS_URL` 或代码内 `redis::Client` |

Sa-Token-Rs 默认将数据保存在内存中，此模式读写速度最快，且避免了序列化与反序列化带来的性能消耗，但是此模式也有一些缺点，比如：

1. 重启后数据会丢失。
2. 无法在分布式环境中共享数据。

为此，Sa-Token-Rs 提供了扩展接口，你可以轻松将会话数据存储在一些专业的缓存中间件上（比如 Redis），
做到重启数据不丢失，而且保证分布式环境下多节点的会话一致性。

---

### 1、Sa-Token-Rs 整合 Redis（`SaTokenDaoRedis`）

对应 Java 的 RedisTemplate 整合方案，Rust 侧使用官方 `redis` 客户端 + `sa-token-dao-redis`：

<!---------------------------- tabs:start ------------------------------>
<!-------- tab:Cargo 方式 -------->
``` toml
[dependencies]
sa-token = { version = "0.1", features = ["redis"] }
# 或直接依赖：
# sa-token-dao-redis = "0.1"
redis = { version = "0.27", features = ["tokio-comp", "connection-manager"] }
tokio = { version = "1", features = ["full"] }
```
<!-------- tab:workspace 方式 -------->
``` toml
# 在 workspace Cargo.toml 中已声明时：
sa-token-dao-redis.workspace = true
redis.workspace = true
```
<!---------------------------- tabs:end ------------------------------>

初始化示例（与 `sa-token-demo-axum-redis` 对齐）：

``` rust
use std::env;
use std::sync::Arc;
use sa_token::prelude::{AsyncSaTokenRuntime, AsyncStpUtil, SaTokenConfig};
use sa_token_core::context::sa_token_context_default_impl::SaTokenContextDefaultImpl;
use sa_token_dao_redis::SaTokenDaoRedis;

/// 连接 Redis 并构建异步运行时
async fn build_runtime() -> AsyncStpUtil {
    let redis_url = env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".into());
    let client = redis::Client::open(redis_url.as_str()).expect("invalid REDIS_URL");
    let dao = SaTokenDaoRedis::connect(client)
        .await
        .expect("redis connect failed");

    let runtime = AsyncSaTokenRuntime::new(
        Arc::new(SaTokenConfig {
            token_name: "satoken".into(),
            timeout: 2_592_000,
            is_concurrent: true,
            is_share: false,
            is_log: true,
            ..Default::default()
        }),
        Arc::new(dao),
        Arc::new(SaTokenContextDefaultImpl),
    );
    AsyncStpUtil::new("login", Arc::new(runtime))
}
```

> [!TIP| label:同步门面说明]
> 内存场景常用同步 `StpUtil` + `SaTokenDaoMemory`。
> Redis DAO 实现的是 `AsyncSaTokenDao`，请优先使用 `AsyncStpUtil`，上层业务 API 语义与 Java 一致。

Redis 的集成有多种方式，缓存的方案也不止 Redis 一种，Sa-Token-Rs 为缓存方案提供多种扩展实现。

如果你对 Sa-Token-Rs 还不太熟悉，或者只想“省心省事”，我们推荐你直接使用上述的 Redis 集成方案，而不必进行过多研究。到此为止，你可以跳转到下一章节了。

如果你想对缓存方案再进行一下深入探究，那么你可以参考：[缓存层扩展](/use/dao-extend)


### 2、自定义序列化方案

如果你按照上述 Redis 方案进行集成测试，会发现框架在 Redis 中是以 **JSON** 格式存储数据的（`serde_json`）。可以自定义数据序列化格式吗？当然是可以的。

框架的默认序列化层调用为 `String 序列化` -> `JSON 序列化`。要自定义数据序列化方式你可以从这两方面入手：


#### 2.1、自定义 JSON 序列化方案：

先说较为底层的 `JSON 序列化`。在 Rust 中，`sa-token-dao-redis` 默认使用 `serde` + `serde_json`（对应 Java 侧 Jackson）。

如果你想更换为其它 JSON 解析框架，可以：

<!------------------------------ tabs:start ------------------------------>

<!------------- tab:serde_json（默认） ------------->
``` toml
# 默认已内置，无需额外依赖
# Session / Object 经 serde_json::to_string / from_str 读写
```

<!------------- tab:自定义 DAO ------------->
``` rust
// 实现 AsyncSaTokenDao（或同步 SaTokenDao），在 get_session / set_session 中
// 使用你偏好的序列化库（simd-json、rmp-serde 等），再注入运行时即可。
// 详细接口见：[缓存层扩展](/use/dao-extend)
```

<!------------- tab:Java 对照：Fastjson ------------->
``` xml
<!-- Java 原版可引入 sa-token-fastjson 等；Rust 侧无同名插件包，请用自定义 DAO 等价实现 -->
```

<!---------------------------- tabs:end ------------------------------>

完整插件列表请参考：[JSON 序列化扩展](/plugin/json-extend)（若章节尚未完全移植，以源码为准）

#### 2.2、自定义 String 序列化方案：

或者你想更直接点，不使用 json 序列化方案，也是可以的。你可以直接自定义数据的 String 序列化方案：

<!------------------------------ tabs:start ------------------------------>

<!------------- tab:自定义 AsyncSaTokenDao ------------->
``` rust
// 实现 AsyncSaTokenDao 的 get/set/update，自行决定 value 的编解码
// （例如 MessagePack、Protobuf、自有二进制协议）
```

<!------------- tab:Java 对照：jdk 序列化 (base64) ------------->
``` java
// Java 原版：
// SaManager.setSaSerializerTemplate(new SaSerializerTemplateForJdkUseBase64());
```

<!------------- tab:Java 对照：jdk 序列化 (16进制) ------------->
``` java
// SaManager.setSaSerializerTemplate(new SaSerializerTemplateForJdkUseHex());
```

<!------------- tab:Java 对照：jdk 序列化 (ISO-8859-1) ------------->
``` java
// SaManager.setSaSerializerTemplate(new SaSerializerTemplateForJdkUseISO_8859_1());
```
<!---------------------------- tabs:end ------------------------------>

除了以上的几种序列化方案，我们还提供了序列化扩展包，详细可参考：[序列化插件扩展包](/plugin/custom-serializer)


### 3、集成 Redis 请注意：

**1. 引入了依赖，我还需要为 Redis 配置连接信息吗？** <br>
需要！只有项目初始化了正确的 Redis 实例，`Sa-Token-Rs` 才可以使用 Redis 进行数据持久化。参考以下配置：

<!---------------------------- tabs:start ------------------------------>
<!-------- tab:环境变量 -------->
``` bash
# 推荐：与 demo 一致
export REDIS_URL=redis://:password@127.0.0.1:6379/1
```
<!-------- tab:代码内 Client -------->
``` rust
let client = redis::Client::open("redis://127.0.0.1:6379/1")?;
let dao = SaTokenDaoRedis::connect(client).await?;
```
<!-------- tab:Java yml 语义对照 -------->
``` yaml
# Java Spring 原版配置（语义对照，Rust 不直接读此文件）
spring:
  data:
    redis:
      database: 1
      host: 127.0.0.1
      port: 6379
      # password:
      timeout: 10s
```
<!---------------------------- tabs:end ------------------------------>

> [!WARNING| label:小提示 ]
> Java SpringBoot3.x 需将前缀 `spring.redis` 改为 `spring.data.redis`。Rust 侧请直接配置 `REDIS_URL` 或 `redis::Client`。


**2. 集成 Redis 后，是我额外手动保存数据，还是框架自动保存？** <br>
框架自动保存。集成 `Redis` 只需要引入对应依赖并注入 DAO 即可，框架所有上层 API 语义保持不变（同步门面或异步门面择一）。

**3. 集成包版本问题** <br>
`sa-token-dao-redis` 的版本尽量与 `sa-token` / `sa-token-core` 一致，否则可能出现兼容性问题。



### 4、扩展：集成 MongoDB

- [集成 MongoDB 参考一](/up/integ-spring-mongod-1)
- [集成 MongoDB 参考二](/up/integ-spring-mongod-2)

可运行示例：`crates/sa-token-demo/sa-token-demo-axum-redis`、`sa-token-demo-actix-redis`。
