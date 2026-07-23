# 持久层扩展
--- 

> Sa-Token → Sa-Token-Rs；Maven → Cargo；Jackson → serde。

Sa-Token-Rs 默认将会话数据保存在内存中，此模式读写速度最快，且避免了序列化与反序列化带来的性能消耗，但是此模式也有一些缺点，比如：重启后数据会丢失，无法在集群模式下共享数据

为此，Sa-Token-Rs 将数据持久操作全部抽象到 `SaTokenDao` 接口中，保证大家对框架进行灵活扩展，比如我们可以将会话数据存储在 `Redis`、`Memcached`等专业的缓存中间件中，做到重启数据不丢失，而且保证分布式环境下多节点的会话一致性

除了框架内部对`SaTokenDao`提供的基于内存的默认实现（`sa-token-dao-memory`），官方仓库还提供了以下扩展方案：<br>


### 1. Sa-Token-Rs 整合 Redis（内存 DAO 对照 / 默认序列化）

Java 原版曾提供「JDK 默认序列化」的 `sa-token-redis`。Rust 侧统一使用 **serde** 序列化，推荐直接使用下一节的 Redis DAO。

| Java | Rust |
|---|---|
| `sa-token-redis`（JDK 序列化） | 不推荐单独对应；请用 `sa-token-dao-redis` + serde |
| 优点：兼容性好 | — |
| 缺点：不可读 | — |


### 2. Sa-Token-Rs 整合 Redis（推荐）

``` toml
[dependencies]
sa-token-dao-redis = "0.1"
# 以及 redis 客户端相关依赖（以实现为准）
```

| Java | Rust |
|---|---|
| `sa-token-redis-template` | `sa-token-dao-redis` |
| Jackson JSON | serde + serde_json |

优点：Session 序列化后可读性强（JSON），可灵活手动修改

注册示例：

``` rust
use std::sync::Arc;
use sa_token::prelude::*;
// use sa_token_dao_redis::SaTokenDaoRedis;

async fn use_redis_dao(/* client */) {
    // let dao = SaTokenDaoRedis::connect(client).await?;
    // SaManager::set_sa_token_dao(Arc::new(dao));
}
```


<br>

### 集成 Redis 请注意：


**1. 无论使用哪种序列化方式，你都必须为项目提供一个 Redis 连接方案，例如：**

``` toml
# 在 Cargo.toml / 运行环境中配置 redis URL，或使用连接池 crate
# 具体依赖以 sa-token-dao-redis 的 README / Cargo.toml 为准
```

**2. 引入了依赖，我还需要为 Redis 配置连接信息吗？** <br>
需要！只有项目初始化了正确的 Redis 客户端，`Sa-Token-Rs` 才可以使用 Redis 进行数据持久化。参考（语义对照 Java yml）：

``` toml
# 环境变量或配置文件示意
# REDIS_URL = "redis://127.0.0.1:6379/1"
```

Java 原文 `spring.redis.*` 对照：

``` yaml
# Java Spring 配置（仅对照）
spring:
  redis:
    database: 1
    host: 127.0.0.1
    port: 6379
    timeout: 1000ms
```


**3. 集成 Redis 后，是我额外手动保存数据，还是框架自动保存？** <br>
框架自动保存。集成 Redis 只需接入对应 DAO 并 `SaManager::set_sa_token_dao(...)`，框架所有上层 API 保持不变。


<br><br>
更多框架的集成方案正在更新中... (欢迎大家提交 PR)

详见：[集成 Redis](/up/integ-redis)
