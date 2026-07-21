//! Redis DAO 实现（基于 redis-rs crate）。
//!
//! 对应 Java Sa-Token 的 Redis 插件，提供基于 Redis 的持久化实现。
//!
//! # 示例
//!
//! ```rust,ignore
//! use sa_token_dao_redis::SaTokenDaoRedis;
//!
//! #[tokio::main]
//! async fn main() {
//!     let client = redis::Client::open("redis://127.0.0.1:6379").unwrap();
//!     let dao = SaTokenDaoRedis::new(client);
//!     // 使用 dao
//! }
//! ```

use redis::Commands;
use sa_token_core::dao::sa_token_dao::SaTokenDao;
use sa_token_core::session::sa_session::SaSession;

/// Redis DAO 实现
pub struct SaTokenDaoRedis {
    /// Redis 客户端
    client: redis::Client,
}

impl SaTokenDaoRedis {
    /// 创建新的 Redis DAO
    pub fn new(client: redis::Client) -> Self {
        Self { client }
    }

    /// 获取 Redis 连接
    fn get_connection(&self) -> redis::RedisResult<redis::Connection> {
        self.client.get_connection()
    }
}

impl SaTokenDao for SaTokenDaoRedis {
    fn get(&self, key: &str) -> Option<String> {
        let mut conn = self.get_connection().ok()?;
        conn.get(key).ok()
    }

    fn set(&self, key: &str, value: &str, timeout: i64) {
        if let Ok(mut conn) = self.get_connection() {
            if timeout > 0 {
                let _: redis::RedisResult<()> = conn.set_ex(key, value, timeout as u64);
            } else {
                let _: redis::RedisResult<()> = conn.set(key, value);
            }
        }
    }

    fn update(&self, key: &str, value: &str) {
        if let Ok(mut conn) = self.get_connection() {
            let _: redis::RedisResult<()> = conn.set(key, value);
        }
    }

    fn delete(&self, key: &str) {
        if let Ok(mut conn) = self.get_connection() {
            let _: redis::RedisResult<i32> = conn.del(key);
        }
    }

    fn get_timeout(&self, key: &str) -> i64 {
        let Ok(mut conn) = self.get_connection() else {
            return -2;
        };
        conn.ttl(key).unwrap_or(-2)
    }

    fn update_timeout(&self, key: &str, timeout: i64) {
        if let Ok(mut conn) = self.get_connection() {
            if timeout <= 0 {
                let _: redis::RedisResult<bool> = conn.persist(key);
            } else {
                let _: redis::RedisResult<bool> = conn.expire(key, timeout as i64);
            }
        }
    }

    fn get_object(&self, key: &str) -> Option<serde_json::Value> {
        let value: String = self.get(key)?;
        serde_json::from_str(&value).ok()
    }

    fn set_object(&self, key: &str, value: &serde_json::Value, timeout: i64) {
        let json_str = serde_json::to_string(value).unwrap_or_default();
        self.set(key, &json_str, timeout);
    }

    fn update_object(&self, key: &str, value: &serde_json::Value) {
        let json_str = serde_json::to_string(value).unwrap_or_default();
        self.update(key, &json_str);
    }

    fn delete_object(&self, key: &str) {
        self.delete(key);
    }

    fn get_object_timeout(&self, key: &str) -> i64 {
        self.get_timeout(key)
    }

    fn update_object_timeout(&self, key: &str, timeout: i64) {
        self.update_timeout(key, timeout);
    }

    fn get_session(&self, session_id: &str) -> Option<SaSession> {
        let value: String = self.get(session_id)?;
        serde_json::from_str(&value).ok()
    }

    fn set_session(&self, session: &SaSession, timeout: i64) {
        let json_str = serde_json::to_string(session).unwrap_or_default();
        self.set(session.id(), &json_str, timeout);
    }

    fn update_session(&self, session: &SaSession) {
        let json_str = serde_json::to_string(session).unwrap_or_default();
        self.update(session.id(), &json_str);
    }

    fn delete_session(&self, session_id: &str) {
        self.delete(session_id);
    }

    fn get_session_timeout(&self, session_id: &str) -> i64 {
        self.get_timeout(session_id)
    }

    fn update_session_timeout(&self, session_id: &str, timeout: i64) {
        self.update_timeout(session_id, timeout);
    }

    fn search_data(
        &self,
        prefix: &str,
        keyword: &str,
        start: i64,
        size: i64,
        sort_type: bool,
    ) -> Vec<String> {
        let Ok(mut conn) = self.get_connection() else {
            return Vec::new();
        };

        let pattern = format!("{}*{}*", prefix, keyword);
        let keys: Vec<String> = redis::cmd("KEYS")
            .arg(&pattern)
            .query(&mut conn)
            .unwrap_or_default();

        let mut results: Vec<String> = keys
            .into_iter()
            .filter(|k| k.starts_with(prefix) && k.contains(keyword))
            .collect();

        if sort_type {
            results.sort();
        } else {
            results.sort_by(|a, b| b.cmp(a));
        }

        let start = start.max(0) as usize;
        let size = size.max(0) as usize;
        results.into_iter().skip(start).take(size).collect()
    }
}
