# Sa-Token-Rs 集成 MongoDB（方案二）
--- 

> Sa-Token → Sa-Token-Rs。本篇对应 Java「Spring Boot + 自定义 MySaSession + SaTokenWrap」另一套 Mongo 集成写法。Rust 侧用 `mongodb` + `serde`；`SaSession` 已可直接 serde，通常**不必**再造 MySaSession。

| Java | Rust |
|---|---|
| `spring-boot-starter-data-mongodb` | `mongodb`（Cargo） |
| `MySaSession extends SaSession` | 一般直接用 `SaSession`（已 `Serialize`/`Deserialize`） |
| `SaStrategy.instance.createSession` | 若需自定义 Session，可在创建处 `SaSession::new` 后包装 |
| `SaTokenDaoMongo` | `impl SaTokenDao for SaTokenDaoMongo` |
| `SaFoxUtil.searchList` | `sa_token_core::util::sa_fox_util::SaFoxUtil::search_list` |

在 axum / actix 应用下集成 MongoDB：

<!---------------------------- tabs:start ------------------------------>
<!-------- tab:Cargo 方式 -------->
``` toml
[dependencies]
sa-token-core = "0.1"
mongodb = { version = "3", features = ["sync"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
```
<!-------- tab:workspace 方式 -------->
``` toml
mongodb.workspace = true
serde.workspace = true
```
<!---------------------------- tabs:end ------------------------------>

1. （可选）自定义 Session  
Java 中因 `SaSession.dataMap` 无 setter，Spring Data 反序列化会失败，故需要 `MySaSession`。  
Rust 的 `SaSession` 已带 `serde` 支持，多数情况可直接存取：

``` rust
use sa_token_core::session::sa_session::SaSession;

/// 若确需扩展字段，可新包装类型（一般不必须）
pub struct MySaSession {
    pub inner: SaSession,
}

impl MySaSession {
    /// 创建自定义 Session
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            inner: SaSession::new(id),
        }
    }
}
```

若业务强制使用自定义类型，在写入 DAO 前把 `MySaSession` 转成可序列化结构即可（对应 Java 重写 `SaStrategy.createSession`）。

2. 在启动入口注册 DAO（对应 Java `main` 里改 Strategy）

``` rust
use std::sync::Arc;
use sa_token::prelude::*;

fn main() {
    // 注册 Mongo DAO（见下文 SaTokenDaoMongo）
    // SaManager::set_sa_token_dao(Arc::new(dao));

    // 再启动 axum / actix ...
}
```

3. 实现 SaTokenDao 接口

``` rust
use chrono::{TimeZone, Utc};
use mongodb::bson::{doc, DateTime as BsonDateTime};
use mongodb::sync::Collection;
use serde::{Deserialize, Serialize};
use sa_token_core::dao::sa_token_dao::{SaTokenDao, NEVER_EXPIRE, NOT_VALUE_EXPIRE};
use sa_token_core::exception::{SaResult, SaTokenException};
use sa_token_core::session::sa_session::SaSession;
use sa_token_core::util::sa_fox_util::SaFoxUtil;

/// 用于保存 Sa-Token 数据的包装文档
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaTokenWrap {
    #[serde(rename = "_id")]
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub object: Option<serde_json::Value>,
    /// TTL 字段：为 None 视为永不删除
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout: Option<BsonDateTime>,
}

impl SaTokenWrap {
    /// 是否仍在有效期内
    pub fn live(&self) -> bool {
        match self.timeout {
            None => true,
            Some(t) => {
                let millis = t.timestamp_millis();
                Utc::now().timestamp_millis() < millis
            }
        }
    }
}

/// SaTokenDao Mongo 实现（方案二）
pub struct SaTokenDaoMongo {
    collection: Collection<SaTokenWrap>,
}

impl SaTokenDaoMongo {
    fn get_by_key(&self, key: &str) -> SaResult<Option<SaTokenWrap>> {
        let wrap = self
            .collection
            .find_one(doc! { "_id": key })
            .map_err(|e| SaTokenException::other(e.to_string()))?;
        Ok(wrap.filter(|w| w.live()))
    }

    fn timeout_to_date(timeout: i64) -> BsonDateTime {
        let millis = timeout * 1000 + Utc::now().timestamp_millis();
        BsonDateTime::from_millis(millis)
    }

    fn upsert_by_path(
        &self,
        key: &str,
        path: &str,
        value: mongodb::bson::Bson,
        timeout: i64,
    ) -> SaResult<()> {
        if timeout == 0 || timeout <= NOT_VALUE_EXPIRE {
            return Ok(());
        }
        let mut set_doc = doc! { path: value };
        let mut unset_doc = doc! {};
        if timeout != NEVER_EXPIRE {
            set_doc.insert("timeout", Self::timeout_to_date(timeout));
        } else {
            unset_doc.insert("timeout", "");
        }
        let mut update = doc! { "$set": set_doc };
        if !unset_doc.is_empty() {
            update.insert("$unset", unset_doc);
        }
        self.collection
            .update_one(doc! { "_id": key }, update)
            .upsert(true)
            .run()
            .map_err(|e| SaTokenException::other(e.to_string()))?;
        Ok(())
    }

    fn update_by_path(
        &self,
        key: &str,
        path: &str,
        value: mongodb::bson::Bson,
    ) -> SaResult<()> {
        let now = BsonDateTime::from_millis(Utc::now().timestamp_millis());
        self.collection
            .update_one(
                doc! { "_id": key, "timeout": { "$gte": now } },
                doc! { "$set": { path: value } },
            )
            .run()
            .map_err(|e| SaTokenException::other(e.to_string()))?;
        Ok(())
    }
}

impl SaTokenDao for SaTokenDaoMongo {
    // ------------------------ String 读写操作

    fn get(&self, key: &str) -> SaResult<Option<String>> {
        Ok(self.get_by_key(key)?.and_then(|w| w.value))
    }

    fn set(&self, key: &str, value: &str, timeout: i64) -> SaResult<()> {
        self.upsert_by_path(key, "value", value.into(), timeout)
    }

    fn update(&self, key: &str, value: &str) -> SaResult<()> {
        self.update_by_path(key, "value", value.into())
    }

    fn delete(&self, key: &str) -> SaResult<()> {
        self.collection
            .delete_one(doc! { "_id": key })
            .map_err(|e| SaTokenException::other(e.to_string()))?;
        Ok(())
    }

    fn get_timeout(&self, key: &str) -> SaResult<i64> {
        let wrap = self
            .collection
            .find_one(doc! { "_id": key })
            .map_err(|e| SaTokenException::other(e.to_string()))?;
        let Some(token_wrap) = wrap else {
            return Ok(NOT_VALUE_EXPIRE);
        };
        let Some(timeout) = token_wrap.timeout else {
            return Ok(NEVER_EXPIRE);
        };
        let remain = (timeout.timestamp_millis() - Utc::now().timestamp_millis()) / 1000;
        if remain < 0 {
            let _ = self.delete(key);
            return Ok(NOT_VALUE_EXPIRE);
        }
        Ok(remain)
    }

    fn update_timeout(&self, key: &str, timeout: i64) -> SaResult<()> {
        let update = if timeout == NEVER_EXPIRE {
            doc! { "$unset": { "timeout": "" } }
        } else {
            doc! { "$set": { "timeout": Self::timeout_to_date(timeout) } }
        };
        self.collection
            .update_one(doc! { "_id": key }, update)
            .upsert(true)
            .run()
            .map_err(|e| SaTokenException::other(e.to_string()))?;
        Ok(())
    }

    // ------------------------ Object 读写操作

    fn get_object(&self, key: &str) -> SaResult<Option<serde_json::Value>> {
        Ok(self.get_by_key(key)?.and_then(|w| w.object))
    }

    fn set_object(&self, key: &str, object: &serde_json::Value, timeout: i64) -> SaResult<()> {
        let bson = mongodb::bson::to_bson(object).map_err(|e| SaTokenException::other(e.to_string()))?;
        self.upsert_by_path(key, "object", bson, timeout)
    }

    fn update_object(&self, key: &str, object: &serde_json::Value) -> SaResult<()> {
        let bson = mongodb::bson::to_bson(object).map_err(|e| SaTokenException::other(e.to_string()))?;
        self.update_by_path(key, "object", bson)
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

    // ------------------------ Session 读写（可走默认对象通道或专用字段）

    fn get_session(&self, session_id: &str) -> SaResult<Option<SaSession>> {
        match self.get_object(session_id)? {
            Some(v) => Ok(serde_json::from_value(v).ok()),
            None => Ok(None),
        }
    }

    fn set_session(&self, session: &SaSession, timeout: i64) -> SaResult<()> {
        let v = serde_json::to_value(session).map_err(|e| SaTokenException::other(e.to_string()))?;
        self.set_object(session.id(), &v, timeout)
    }

    fn update_session(&self, session: &SaSession) -> SaResult<()> {
        let v = serde_json::to_value(session).map_err(|e| SaTokenException::other(e.to_string()))?;
        self.update_object(session.id(), &v)
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

    // --------- 会话管理 / 搜索

    fn search_data(
        &self,
        prefix: &str,
        keyword: &str,
        start: i64,
        size: i64,
        sort_type: bool,
    ) -> SaResult<Vec<String>> {
        let now = BsonDateTime::from_millis(Utc::now().timestamp_millis());
        let pattern = format!("{prefix}*{keyword}*");
        // Mongo 正则请按实际转义；此处保持与 Java 示例相近的示意
        let cursor = self
            .collection
            .find(doc! {
                "_id": { "$regex": pattern },
                "timeout": { "$gte": now }
            })
            .run()
            .map_err(|e| SaTokenException::other(e.to_string()))?;

        let mut list: Vec<String> = Vec::new();
        for item in cursor {
            let wrap = item.map_err(|e| SaTokenException::other(e.to_string()))?;
            if let Some(v) = wrap.value {
                if !v.is_empty() {
                    list.push(v);
                }
            }
        }
        Ok(SaFoxUtil::search_list(&mut list, start as i32, size as i32, sort_type))
    }
}
```

> [!WARNING| label:注意]
> 1. 请为 `timeout` 字段创建 TTL 索引，使过期文档由 MongoDB 自动清理。  
> 2. 示例中的 `mongodb` sync API 调用以教学为主，升级依赖后请对照官方文档微调。  
> 3. 无官方 `sa-token-dao-mongodb` 时，本文件仅作扩展参考；稳定方案仍推荐内存 / Redis DAO。
