//! Asynchronous Redis storage adapter.

use async_trait::async_trait;
use futures_util::StreamExt;
use redis::aio::{ConnectionManager, ConnectionManagerConfig};
use redis::{AsyncCommands, Script};
use sa_token_core::dao::AsyncSaTokenDao;
use sa_token_core::exception::{SaResult, SaTokenException};
use sa_token_core::session::sa_session::SaSession;
use std::time::Duration;

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

/// Redis-backed asynchronous DAO using a reconnecting connection manager.
#[derive(Clone)]
pub struct SaTokenDaoRedis {
    manager: ConnectionManager,
}

impl SaTokenDaoRedis {
    /// Connects to Redis with bounded connection and response timeouts.
    ///
    /// # Errors
    ///
    /// Returns an error when the initial Redis connection cannot be established.
    pub async fn connect(client: redis::Client) -> SaResult<Self> {
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
        Ok(Self { manager })
    }

    /// Creates an adapter from an already configured connection manager.
    pub fn from_manager(manager: ConnectionManager) -> Self {
        Self { manager }
    }

    fn connection(&self) -> ConnectionManager {
        self.manager.clone()
    }
}

fn redis_error(error: redis::RedisError) -> SaTokenException {
    SaTokenException::Other {
        message: format!("Redis operation failed: {error}"),
    }
}

fn serialization_error(error: serde_json::Error) -> SaTokenException {
    SaTokenException::Other {
        message: format!("Redis value serialization failed: {error}"),
    }
}

#[async_trait]
impl AsyncSaTokenDao for SaTokenDaoRedis {
    async fn get(&self, key: &str) -> SaResult<Option<String>> {
        self.connection().get(key).await.map_err(redis_error)
    }

    async fn set(&self, key: &str, value: &str, timeout: i64) -> SaResult<()> {
        let mut connection = self.connection();
        if timeout > 0 {
            connection
                .set_ex::<_, _, ()>(key, value, timeout as u64)
                .await
                .map_err(redis_error)
        } else {
            connection
                .set::<_, _, ()>(key, value)
                .await
                .map_err(redis_error)
        }
    }

    async fn update(&self, key: &str, value: &str) -> SaResult<()> {
        Script::new(UPDATE_KEEP_TTL_SCRIPT)
            .key(key)
            .arg(value)
            .invoke_async::<i64>(&mut self.connection())
            .await
            .map(|_| ())
            .map_err(redis_error)
    }

    async fn delete(&self, key: &str) -> SaResult<()> {
        self.connection()
            .del::<_, usize>(key)
            .await
            .map(|_| ())
            .map_err(redis_error)
    }

    async fn get_timeout(&self, key: &str) -> SaResult<i64> {
        self.connection().ttl(key).await.map_err(redis_error)
    }

    async fn update_timeout(&self, key: &str, timeout: i64) -> SaResult<()> {
        let mut connection = self.connection();
        if timeout <= 0 {
            connection
                .persist::<_, bool>(key)
                .await
                .map(|_| ())
                .map_err(redis_error)
        } else {
            connection
                .expire::<_, bool>(key, timeout)
                .await
                .map(|_| ())
                .map_err(redis_error)
        }
    }

    async fn get_object(&self, key: &str) -> SaResult<Option<serde_json::Value>> {
        self.get(key)
            .await?
            .map(|value| serde_json::from_str(&value).map_err(serialization_error))
            .transpose()
    }

    async fn set_object(&self, key: &str, value: &serde_json::Value, timeout: i64) -> SaResult<()> {
        let encoded = serde_json::to_string(value).map_err(serialization_error)?;
        self.set(key, &encoded, timeout).await
    }

    async fn update_object(&self, key: &str, value: &serde_json::Value) -> SaResult<()> {
        let encoded = serde_json::to_string(value).map_err(serialization_error)?;
        self.update(key, &encoded).await
    }

    async fn get_session(&self, session_id: &str) -> SaResult<Option<SaSession>> {
        self.get(session_id)
            .await?
            .map(|value| serde_json::from_str(&value).map_err(serialization_error))
            .transpose()
    }

    async fn set_session(&self, session: &SaSession, timeout: i64) -> SaResult<()> {
        let encoded = serde_json::to_string(session).map_err(serialization_error)?;
        self.set(session.id(), &encoded, timeout).await
    }

    async fn update_session(&self, session: &SaSession) -> SaResult<()> {
        let encoded = serde_json::to_string(session).map_err(serialization_error)?;
        self.update(session.id(), &encoded).await
    }

    async fn search_data(
        &self,
        prefix: &str,
        keyword: &str,
        start: i64,
        size: i64,
        ascending: bool,
    ) -> SaResult<Vec<String>> {
        let pattern = format!("{prefix}*{keyword}*");
        let mut connection = self.connection();
        let iterator = connection
            .scan_match::<_, String>(&pattern)
            .await
            .map_err(redis_error)?;
        let values: Vec<redis::RedisResult<String>> = iterator.collect().await;
        let mut keys = values
            .into_iter()
            .collect::<redis::RedisResult<Vec<_>>>()
            .map_err(redis_error)?;

        if ascending {
            keys.sort();
        } else {
            keys.sort_by(|left, right| right.cmp(left));
        }

        let start = start.max(0) as usize;
        let size = if size < 0 { usize::MAX } else { size as usize };
        Ok(keys.into_iter().skip(start).take(size).collect())
    }
}
