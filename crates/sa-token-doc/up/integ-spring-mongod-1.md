# Sa-Token-Rs 集成 MongoDB 
--- 

> Sa-Token → Sa-Token-Rs。Java Spring Data MongoDB → Rust `mongodb` crate + 自定义 `SaTokenDao`。官方暂无 `sa-token-dao-mongodb` 开箱 crate，本篇演示自行扩展。

| Java | Rust |
|---|---|
| `spring-boot-starter-data-mongodb` | `mongodb` crate（Cargo） |
| `MongoTemplate` | `mongodb::Collection` / `Database` |
| `implements SaTokenDao` | `impl SaTokenDao for SaTokenMongoDao` |
| `@Document` + TTL 索引 | BSON 文档 + TTL index |
| Jackson | `serde` / `serde_json` |

此章介绍如何通过扩展 `SaTokenDao` 接口来实现 MongoDB 的集成。

Java 社区示例可参考：[sa-token-mongodb-demo](https://gitee.com/lilihao/sa-token-mongodb-demo)（Spring Boot）。Rust 侧思路相同：实现持久化 trait，并在启动时 `SaManager::set_sa_token_dao(...)`。

先决条件：
1. Tokio 异步运行时（若 DAO 内部用异步 client，可用 `block_on` 包装同步 `SaTokenDao`，或优先使用 `AsyncSaTokenDao` + `AsyncStpUtil`）
2. `mongodb` crate

以下是依赖的引入：

---


<!---------------------------- tabs:start ------------------------------>
<!-------- tab:Cargo 方式 -------->
``` toml
[dependencies]
sa-token-core = "0.1"
mongodb = { version = "3", features = ["sync"] }  # 或异步 features
serde = { version = "1", features = ["derive"] }
serde_json = "1"
chrono = { version = "0.4", features = ["serde"] }
```
<!-------- tab:workspace 方式 -------->
``` toml
# workspace Cargo.toml
mongodb.workspace = true
serde.workspace = true
```
<!---------------------------- tabs:end ------------------------------>

优点：少量改造即可完成集成 MongoDB




### 集成代码：


**1. 创建一个结构体来包装 Sa-Token-Rs 的数据**
``` rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sa_token_core::session::sa_session::SaSession;

/// MongoDB 中保存的 Sa-Token 数据文档
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaTokenMongoData {
    /// 文档 id（可与 key 相同）
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,

    /// token / 业务 key（唯一）
    pub key: String,

    /// sa-token 的 session（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session: Option<SaSession>,

    /// sa-token 的 token string（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub string: Option<String>,

    /// 过期时间；为 None 表示永不过期。配合 MongoDB TTL 索引自动删除
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expire_at: Option<DateTime<Utc>>,
}
```

在 MongoDB 上为 `expire_at` 创建 TTL 索引（`expireAfterSeconds = 0`），过期文档由服务端自动清理。

**2.实现 SaTokenDao**

这个 `SaTokenMongoDao` 是仿照官方的 redis 集成实现的（同步示意；生产可改为 `AsyncSaTokenDao`）：

``` rust
use std::sync::Arc;
use chrono::{Duration, Utc};
use mongodb::bson::doc;
use mongodb::sync::{Client, Collection};
use sa_token_core::dao::sa_token_dao::{self, SaTokenDao, NEVER_EXPIRE, NOT_VALUE_EXPIRE};
use sa_token_core::exception::{SaResult, SaTokenException};
use sa_token_core::session::sa_session::SaSession;
use sa_token_core::sa_manager::SaManager;

use super::SaTokenMongoData;

/// MongoDB 持久化实现（对应 Java SaTokenMongoDao）
pub struct SaTokenMongoDao {
    collection: Collection<SaTokenMongoData>,
}

impl SaTokenMongoDao {
    /// 连接并构造 DAO
    pub fn connect(uri: &str, db: &str, coll: &str) -> SaResult<Self> {
        let client = Client::with_uri_str(uri).map_err(|e| SaTokenException::other(e.to_string()))?;
        let collection = client.database(db).collection(coll);
        // 建议在此确保 key 唯一索引 + expire_at TTL 索引
        Ok(Self { collection })
    }

    fn expire_at_from_timeout(timeout: i64) -> Option<chrono::DateTime<Utc>> {
        // NEVER_EXPIRE => None，MongoDB 不会按 TTL 删除
        if timeout == NEVER_EXPIRE {
            None
        } else {
            Some(Utc::now() + Duration::seconds(timeout))
        }
    }
}

impl SaTokenDao for SaTokenMongoDao {
    fn get(&self, key: &str) -> SaResult<Option<String>> {
        let filter = doc! { "key": key };
        let found = self
            .collection
            .find_one(filter)
            .map_err(|e| SaTokenException::other(e.to_string()))?;
        Ok(found.and_then(|d| d.string))
    }

    fn set(&self, key: &str, value: &str, timeout: i64) -> SaResult<()> {
        if timeout == 0 || timeout <= NOT_VALUE_EXPIRE {
            return Ok(());
        }
        let filter = doc! { "key": key };
        let update = doc! {
            "$set": {
                "string": value,
                "expire_at": Self::expire_at_from_timeout(timeout)
                    .map(|t| mongodb::bson::DateTime::from_millis(t.timestamp_millis())),
            }
        };
        self.collection
            .update_one(filter, update)
            .upsert(true)
            .run()
            .map_err(|e| SaTokenException::other(e.to_string()))?;
        Ok(())
    }

    fn update(&self, key: &str, value: &str) -> SaResult<()> {
        let expire = self.get_timeout(key)?;
        if expire == NOT_VALUE_EXPIRE {
            return Ok(());
        }
        self.set(key, value, expire)
    }

    fn delete(&self, key: &str) -> SaResult<()> {
        self.collection
            .delete_one(doc! { "key": key })
            .map_err(|e| SaTokenException::other(e.to_string()))?;
        Ok(())
    }

    fn get_timeout(&self, key: &str) -> SaResult<i64> {
        let found = self
            .collection
            .find_one(doc! { "key": key })
            .map_err(|e| SaTokenException::other(e.to_string()))?;
        let Some(data) = found else {
            return Ok(NOT_VALUE_EXPIRE);
        };
        let Some(expire_at) = data.expire_at else {
            return Ok(NEVER_EXPIRE);
        };
        let seconds = (expire_at - Utc::now()).num_seconds();
        Ok(if seconds < 0 { 0 } else { seconds })
    }

    fn update_timeout(&self, key: &str, timeout: i64) -> SaResult<()> {
        if timeout == NEVER_EXPIRE {
            let expire = self.get_timeout(key)?;
            if expire != NEVER_EXPIRE {
                if let Some(v) = self.get(key)? {
                    self.set(key, &v, timeout)?;
                }
            }
            return Ok(());
        }
        let filter = doc! { "key": key };
        let update = doc! {
            "$set": {
                "expire_at": Self::expire_at_from_timeout(timeout)
                    .map(|t| mongodb::bson::DateTime::from_millis(t.timestamp_millis())),
            }
        };
        self.collection
            .update_one(filter, update)
            .upsert(true)
            .run()
            .map_err(|e| SaTokenException::other(e.to_string()))?;
        Ok(())
    }

    fn get_object(&self, key: &str) -> SaResult<Option<serde_json::Value>> {
        let found = self
            .collection
            .find_one(doc! { "key": key })
            .map_err(|e| SaTokenException::other(e.to_string()))?;
        Ok(found.and_then(|d| d.session.map(|s| serde_json::to_value(s).ok()).flatten()))
    }

    fn set_object(&self, key: &str, value: &serde_json::Value, timeout: i64) -> SaResult<()> {
        if timeout == 0 || timeout <= NOT_VALUE_EXPIRE {
            return Ok(());
        }
        let session: SaSession =
            serde_json::from_value(value.clone()).map_err(|e| SaTokenException::other(e.to_string()))?;
        let filter = doc! { "key": key };
        let update = doc! {
            "$set": {
                "session": mongodb::bson::to_bson(&session).map_err(|e| SaTokenException::other(e.to_string()))?,
                "expire_at": Self::expire_at_from_timeout(timeout)
                    .map(|t| mongodb::bson::DateTime::from_millis(t.timestamp_millis())),
            }
        };
        self.collection
            .update_one(filter, update)
            .upsert(true)
            .run()
            .map_err(|e| SaTokenException::other(e.to_string()))?;
        Ok(())
    }

    fn update_object(&self, key: &str, value: &serde_json::Value) -> SaResult<()> {
        let expire = self.get_object_timeout(key)?;
        if expire == NOT_VALUE_EXPIRE {
            return Ok(());
        }
        self.set_object(key, value, expire)
    }

    fn delete_object(&self, key: &str) -> SaResult<()> {
        self.delete(key)
    }

    fn get_object_timeout(&self, key: &str) -> SaResult<i64> {
        self.get_timeout(key)
    }

    fn update_object_timeout(&self, key: &str, timeout: i64) -> SaResult<()> {
        self.update_timeout(key, timeout)
    }

    fn get_session(&self, session_id: &str) -> SaResult<Option<SaSession>> {
        let found = self
            .collection
            .find_one(doc! { "key": session_id })
            .map_err(|e| SaTokenException::other(e.to_string()))?;
        Ok(found.and_then(|d| d.session))
    }

    fn set_session(&self, session: &SaSession, timeout: i64) -> SaResult<()> {
        let value = serde_json::to_value(session).map_err(|e| SaTokenException::other(e.to_string()))?;
        self.set_object(session.id(), &value, timeout)
    }

    fn update_session(&self, session: &SaSession) -> SaResult<()> {
        let value = serde_json::to_value(session).map_err(|e| SaTokenException::other(e.to_string()))?;
        self.update_object(session.id(), &value)
    }

    fn delete_session(&self, session_id: &str) -> SaResult<()> {
        self.delete(session_id)
    }

    fn get_session_timeout(&self, session_id: &str) -> SaResult<i64> {
        self.get_timeout(session_id)
    }

    fn update_session_timeout(&self, session_id: &str, timeout: i64) -> SaResult<()> {
        self.update_timeout(session_id, timeout)
    }

    fn search_data(
        &self,
        prefix: &str,
        keyword: &str,
        start: i64,
        size: i64,
        _sort_type: bool,
    ) -> SaResult<Vec<String>> {
        // 简化：按 key 正则匹配；生产环境请补排序与精确分页
        let mut and_conds = vec![];
        if !prefix.is_empty() {
            and_conds.push(doc! { "key": { "$regex": format!("^{}", regex::escape(prefix)) } });
        }
        if !keyword.is_empty() {
            and_conds.push(doc! { "key": { "$regex": regex::escape(keyword), "$options": "i" } });
        }
        let filter = if and_conds.is_empty() {
            doc! {}
        } else {
            doc! { "$and": and_conds }
        };
        let skip = start.max(0) as u64;
        let limit = if size < 0 { 0 } else { size as i64 };
        let mut cursor = self
            .collection
            .find(filter)
            .skip(skip)
            .limit(limit)
            .run()
            .map_err(|e| SaTokenException::other(e.to_string()))?;
        let mut keys = Vec::new();
        while let Some(doc) = cursor.next() {
            let data = doc.map_err(|e| SaTokenException::other(e.to_string()))?;
            keys.push(data.key);
        }
        Ok(keys)
    }
}

/// 启动时注册 DAO
pub fn init_mongo_dao() -> SaResult<()> {
    let dao = SaTokenMongoDao::connect("mongodb://127.0.0.1:27017", "satoken", "saTokenMongo")?;
    SaManager::set_sa_token_dao(Arc::new(dao));
    Ok(())
}
```

> [!TIP| label:实现提示]
> 上文为教学示意，`mongodb` sync API 方法名可能随版本微调，请以当前 crate 文档为准。若使用纯异步 client，请实现 `AsyncSaTokenDao` 并配合 `AsyncStpUtil`，勿对同步 `StpUtil` 假写 `.await`。

