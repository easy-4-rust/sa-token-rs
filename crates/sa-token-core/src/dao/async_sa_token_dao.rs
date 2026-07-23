//! Object-safe asynchronous persistence port.

use async_trait::async_trait;

use crate::exception::SaResult;
use crate::session::sa_session::SaSession;

/// Asynchronous storage contract used by Redis and asynchronous web runtimes.
///
/// Every operation preserves the distinction between a missing value and an
/// infrastructure failure. Implementations must not block an async runtime worker.
#[async_trait]
pub trait AsyncSaTokenDao: Send + Sync + 'static {
    /// Reads a string value.
    async fn get(&self, key: &str) -> SaResult<Option<String>>;

    /// Writes a string value with a timeout in seconds.
    async fn set(&self, key: &str, value: &str, timeout: i64) -> SaResult<()>;

    /// Updates a string value without changing its remaining TTL.
    async fn update(&self, key: &str, value: &str) -> SaResult<()>;

    /// Deletes a value.
    async fn delete(&self, key: &str) -> SaResult<()>;

    /// Returns the remaining TTL in seconds (`-1` persistent, `-2` missing).
    async fn get_timeout(&self, key: &str) -> SaResult<i64>;

    /// Changes the remaining TTL in seconds.
    async fn update_timeout(&self, key: &str, timeout: i64) -> SaResult<()>;

    /// Reads a JSON value.
    async fn get_object(&self, key: &str) -> SaResult<Option<serde_json::Value>>;

    /// Writes a JSON value.
    async fn set_object(&self, key: &str, value: &serde_json::Value, timeout: i64) -> SaResult<()>;

    /// Updates a JSON value without changing its remaining TTL.
    async fn update_object(&self, key: &str, value: &serde_json::Value) -> SaResult<()>;

    /// Deletes a JSON value.
    async fn delete_object(&self, key: &str) -> SaResult<()> {
        self.delete(key).await
    }

    /// Returns a JSON value's remaining TTL.
    async fn get_object_timeout(&self, key: &str) -> SaResult<i64> {
        self.get_timeout(key).await
    }

    /// Changes a JSON value's remaining TTL.
    async fn update_object_timeout(&self, key: &str, timeout: i64) -> SaResult<()> {
        self.update_timeout(key, timeout).await
    }

    /// Reads a session.
    async fn get_session(&self, session_id: &str) -> SaResult<Option<SaSession>>;

    /// Writes a session.
    async fn set_session(&self, session: &SaSession, timeout: i64) -> SaResult<()>;

    /// Updates a session without changing its remaining TTL.
    async fn update_session(&self, session: &SaSession) -> SaResult<()>;

    /// Deletes a session.
    async fn delete_session(&self, session_id: &str) -> SaResult<()> {
        self.delete(session_id).await
    }

    /// Returns a session's remaining TTL.
    async fn get_session_timeout(&self, session_id: &str) -> SaResult<i64> {
        self.get_timeout(session_id).await
    }

    /// Changes a session's remaining TTL.
    async fn update_session_timeout(&self, session_id: &str, timeout: i64) -> SaResult<()> {
        self.update_timeout(session_id, timeout).await
    }

    /// Searches keys using incremental backend iteration.
    async fn search_data(
        &self,
        prefix: &str,
        keyword: &str,
        start: i64,
        size: i64,
        ascending: bool,
    ) -> SaResult<Vec<String>>;
}
