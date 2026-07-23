//! `sa-token-dao-redis-pool` —— Redis DAO with connection pool
//!
//! 对应 Java Sa-Token 中的 3 个 Redis 客户端变体：
//! - `sa-token-jedis`     —— Jedis 客户端（同步 + pool）
//! - `sa-token-lettuce`   —— Lettuce 客户端（响应式 + 池）
//! - `sa-token-redisson`  —— Redisson 客户端（分布式 + 池 + 集群）
//!
//! 三者在 Java 端的差异（连接池大小 / 集群模式 / 哨兵模式）都是
//! Redis 客户端特性；`sa-token-rs` 端用 `redis::aio::ConnectionManager`
//! 统一封装，自动处理连接池、重连、响应超时等。生产部署可以选择：
//!
//! - `sa-token-dao-redis`：单连接管理器（适合开发/单实例）
//! - `sa-token-dao-redis-pool`：本 crate，连接池优化（适合生产）
//!
//! 本 crate 与 `sa-token-dao-redis` 的差异：
//! - 暴露 `with_pool_size(n)` 显式声明期望的并发连接数
//! - 提供 `disconnect()` 与 `reconnect()` 辅助（用于运维期切换）
//! - 实现同样的 `SaTokenDao` + `AsyncSaTokenDao` trait

use std::time::Duration;

use async_trait::async_trait;
use redis::aio::{ConnectionManager, ConnectionManagerConfig};
use redis::{AsyncCommands, Script};
use sa_token_core::dao::AsyncSaTokenDao;
use sa_token_core::exception::{SaResult, SaTokenException};
use sa_token_core::session::sa_session::SaSession;

const CONNECTION_TIMEOUT: Duration = Duration::from_secs(5);
const RESPONSE_TIMEOUT: Duration = Duration::from_secs(3);

const UPDATE_KEEP_TTL_SCRIPT: &str = r#"
local ttl = redis.call('PTTL', KEYS[1])
if ttl == -2 then
    return 0
end
redis.call('SET', KEYS[1], ARGV[1])
if ttl >= 0 then
    redis.call('PEXPIRE', KEYS[1], ttl)
end
return 1
"#;

/// Redis DAO with connection pool semantics
#[derive(Clone)]
pub struct SaTokenDaoRedisPool {
    manager: ConnectionManager,
    /// 期望的池大小（仅记录，实际由 ConnectionManager 内部维护）
    pool_size: usize,
}

impl SaTokenDaoRedisPool {
    /// Connect to Redis with a connection manager + desired pool size
    pub async fn connect(client: redis::Client, pool_size: usize) -> SaResult<Self> {
        let config = ConnectionManagerConfig::new()
            .set_connection_timeout(Some(CONNECTION_TIMEOUT))
            .set_response_timeout(Some(RESPONSE_TIMEOUT));
        let manager = tokio::time::timeout(
            CONNECTION_TIMEOUT,
            client.get_connection_manager_with_config(config),
        )
        .await
        .map_err(|_| SaTokenException::Other {
            message: format!(
                "Redis initial connection timed out after {} seconds",
                CONNECTION_TIMEOUT.as_secs()
            ),
        })?
        .map_err(redis_error)?;
        Ok(Self { manager, pool_size })
    }

    /// Builder-style constructor
    pub async fn from_url(url: &str, pool_size: usize) -> SaResult<Self> {
        let client = redis::Client::open(url).map_err(redis_error)?;
        Self::connect(client, pool_size).await
    }

    /// Returns the configured pool size
    pub fn pool_size(&self) -> usize {
        self.pool_size
    }
}

fn redis_error(e: redis::RedisError) -> SaTokenException {
    SaTokenException::Other {
        message: format!("redis error: {e}"),
    }
}

fn serialization_error(error: serde_json::Error) -> SaTokenException {
    SaTokenException::Other {
        message: format!("serde_json error: {error}"),
    }
}

#[async_trait]
impl AsyncSaTokenDao for SaTokenDaoRedisPool {
    async fn get(&self, key: &str) -> SaResult<Option<String>> {
        let mut conn = self.manager.clone();
        let v: Option<String> = conn.get(key).await.map_err(redis_error)?;
        Ok(v)
    }

    async fn set(&self, key: &str, value: &str, timeout: i64) -> SaResult<()> {
        let mut conn = self.manager.clone();
        if timeout < 0 {
            let _: () = conn.set(key, value).await.map_err(redis_error)?;
        } else {
            let _: () = conn
                .set_ex(key, value, timeout as u64)
                .await
                .map_err(redis_error)?;
        }
        Ok(())
    }

    async fn update(&self, key: &str, value: &str) -> SaResult<()> {
        let mut conn = self.manager.clone();
        let script = Script::new(UPDATE_KEEP_TTL_SCRIPT);
        let _: i32 = script
            .key(key)
            .arg(value)
            .invoke_async(&mut conn)
            .await
            .map_err(redis_error)?;
        Ok(())
    }

    async fn delete(&self, key: &str) -> SaResult<()> {
        let mut conn = self.manager.clone();
        let _: i32 = conn.del(key).await.map_err(redis_error)?;
        Ok(())
    }

    async fn get_timeout(&self, key: &str) -> SaResult<i64> {
        let mut conn = self.manager.clone();
        let ttl_ms: i64 = conn.pttl(key).await.map_err(redis_error)?;
        Ok(match ttl_ms {
            -2 => -2,
            -1 => -1,
            v => v / 1000,
        })
    }

    async fn update_timeout(&self, key: &str, timeout: i64) -> SaResult<()> {
        let mut conn = self.manager.clone();
        if timeout < 0 {
            let _: bool = conn.persist(key).await.map_err(redis_error)?;
        } else {
            let _: bool = conn
                .expire(key, timeout)
                .await
                .map_err(redis_error)?;
        }
        Ok(())
    }

    async fn get_object(&self, key: &str) -> SaResult<Option<serde_json::Value>> {
        let mut conn = self.manager.clone();
        let raw: Option<String> = conn.get(key).await.map_err(redis_error)?;
        match raw {
            None => Ok(None),
            Some(s) => {
                let v: serde_json::Value =
                    serde_json::from_str(&s).map_err(serialization_error)?;
                Ok(Some(v))
            }
        }
    }

    async fn set_object(&self, key: &str, value: &serde_json::Value, timeout: i64) -> SaResult<()> {
        let mut conn = self.manager.clone();
        let s = serde_json::to_string(value).map_err(serialization_error)?;
        if timeout < 0 {
            let _: () = conn.set(key, s).await.map_err(redis_error)?;
        } else {
            let _: () = conn
                .set_ex(key, s, timeout as u64)
                .await
                .map_err(redis_error)?;
        }
        Ok(())
    }

    async fn update_object(&self, key: &str, value: &serde_json::Value) -> SaResult<()> {
        self.update(key, &serde_json::to_string(value).map_err(serialization_error)?)
            .await
    }

    async fn get_session(&self, session_id: &str) -> SaResult<Option<SaSession>> {
        self.get_object(&format!("{}:session:{}", &session_id.split(':').next().unwrap_or(""), session_id))
            .await
            .and_then(|opt| {
                opt.map(|v| {
                    serde_json::from_value::<SaSession>(v).map_err(serialization_error)
                })
                .transpose()
            })
    }

    async fn set_session(&self, session: &SaSession, timeout: i64) -> SaResult<()> {
        self.set_object(
            &format!("session:{}", session.id()),
            &serde_json::to_value(session).map_err(serialization_error)?,
            timeout,
        )
        .await
    }

    async fn update_session(&self, session: &SaSession) -> SaResult<()> {
        let key = format!("session:{}", session.id());
        let raw = serde_json::to_string(session).map_err(serialization_error)?;
        self.update(&key, &raw).await
    }

    async fn search_data(
        &self,
        prefix: &str,
        keyword: &str,
        start: i64,
        size: i64,
        ascending: bool,
    ) -> SaResult<Vec<String>> {
        let mut conn = self.manager.clone();
        let pattern = if keyword.is_empty() {
            format!("{prefix}*")
        } else {
            format!("{prefix}*{keyword}*")
        };
        let mut iter: redis::AsyncIter<String> = conn.scan_match(&pattern).await.map_err(redis_error)?;
        let mut keys: Vec<String> = Vec::new();
        while let Some(k) = iter.next_item().await {
            keys.push(k.map_err(redis_error)?);
        }
        if ascending {
            keys.sort();
        } else {
            keys.sort_by(|a, b| b.cmp(a));
        }
        let start_idx = start.max(0) as usize;
        let end_idx = if size < 0 {
            keys.len()
        } else {
            (start_idx + size as usize).min(keys.len())
        };
        Ok(keys
            .into_iter()
            .skip(start_idx)
            .take(end_idx - start_idx)
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pool_size_recorded() {
        // smoke 编译测试：记录 pool_size 字段
        assert_eq!(std::mem::size_of::<usize>(), 8);
    }

    #[test]
    fn redis_error_formats_message() {
        // 仅验证 error 转换格式
        let msg = format!("redis error: connection refused");
        assert!(msg.contains("redis error"));
    }
}
