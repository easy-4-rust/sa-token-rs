//! Standalone in-memory DAO adapter.
//!
//! The canonical in-memory state machine lives in `sa-token-core`; this crate
//! exposes it as an opt-in storage adapter without maintaining a divergent copy.

use async_trait::async_trait;
use sa_token_core::dao::async_sa_token_dao::AsyncSaTokenDao;
use sa_token_core::dao::sa_token_dao::SaTokenDao;
use sa_token_core::dao::sa_token_dao_default_impl::SaTokenDaoDefaultImpl;
use sa_token_core::exception::SaResult;
use sa_token_core::session::sa_session::SaSession;

/// In-memory persistence adapter for single-process deployments and tests.
#[derive(Default)]
pub struct SaTokenDaoMemory {
    inner: SaTokenDaoDefaultImpl,
}

impl SaTokenDaoMemory {
    /// Creates an empty in-memory adapter.
    pub fn new() -> Self {
        Self::default()
    }
}

impl SaTokenDao for SaTokenDaoMemory {
    fn get(&self, key: &str) -> SaResult<Option<String>> {
        self.inner.get(key)
    }

    fn set(&self, key: &str, value: &str, timeout: i64) -> SaResult<()> {
        self.inner.set(key, value, timeout)
    }

    fn update(&self, key: &str, value: &str) -> SaResult<()> {
        self.inner.update(key, value)
    }

    fn delete(&self, key: &str) -> SaResult<()> {
        self.inner.delete(key)
    }

    fn get_timeout(&self, key: &str) -> SaResult<i64> {
        self.inner.get_timeout(key)
    }

    fn update_timeout(&self, key: &str, timeout: i64) -> SaResult<()> {
        self.inner.update_timeout(key, timeout)
    }

    fn get_object(&self, key: &str) -> SaResult<Option<serde_json::Value>> {
        self.inner.get_object(key)
    }

    fn set_object(&self, key: &str, value: &serde_json::Value, timeout: i64) -> SaResult<()> {
        self.inner.set_object(key, value, timeout)
    }

    fn update_object(&self, key: &str, value: &serde_json::Value) -> SaResult<()> {
        self.inner.update_object(key, value)
    }

    fn delete_object(&self, key: &str) -> SaResult<()> {
        self.inner.delete_object(key)
    }

    fn get_object_timeout(&self, key: &str) -> SaResult<i64> {
        self.inner.get_object_timeout(key)
    }

    fn update_object_timeout(&self, key: &str, timeout: i64) -> SaResult<()> {
        self.inner.update_object_timeout(key, timeout)
    }

    fn get_session(&self, session_id: &str) -> SaResult<Option<SaSession>> {
        self.inner.get_session(session_id)
    }

    fn set_session(&self, session: &SaSession, timeout: i64) -> SaResult<()> {
        self.inner.set_session(session, timeout)
    }

    fn update_session(&self, session: &SaSession) -> SaResult<()> {
        self.inner.update_session(session)
    }

    fn delete_session(&self, session_id: &str) -> SaResult<()> {
        self.inner.delete_session(session_id)
    }

    fn get_session_timeout(&self, session_id: &str) -> SaResult<i64> {
        self.inner.get_session_timeout(session_id)
    }

    fn update_session_timeout(&self, session_id: &str, timeout: i64) -> SaResult<()> {
        self.inner.update_session_timeout(session_id, timeout)
    }

    fn search_data(
        &self,
        prefix: &str,
        keyword: &str,
        start: i64,
        size: i64,
        sort_type: bool,
    ) -> SaResult<Vec<String>> {
        self.inner
            .search_data(prefix, keyword, start, size, sort_type)
    }
}

#[async_trait]
impl AsyncSaTokenDao for SaTokenDaoMemory {
    async fn get(&self, key: &str) -> SaResult<Option<String>> {
        SaTokenDao::get(self, key)
    }

    async fn set(&self, key: &str, value: &str, timeout: i64) -> SaResult<()> {
        SaTokenDao::set(self, key, value, timeout)
    }

    async fn update(&self, key: &str, value: &str) -> SaResult<()> {
        SaTokenDao::update(self, key, value)
    }

    async fn delete(&self, key: &str) -> SaResult<()> {
        SaTokenDao::delete(self, key)
    }

    async fn get_timeout(&self, key: &str) -> SaResult<i64> {
        SaTokenDao::get_timeout(self, key)
    }

    async fn update_timeout(&self, key: &str, timeout: i64) -> SaResult<()> {
        SaTokenDao::update_timeout(self, key, timeout)
    }

    async fn get_object(&self, key: &str) -> SaResult<Option<serde_json::Value>> {
        SaTokenDao::get_object(self, key)
    }

    async fn set_object(&self, key: &str, value: &serde_json::Value, timeout: i64) -> SaResult<()> {
        SaTokenDao::set_object(self, key, value, timeout)
    }

    async fn update_object(&self, key: &str, value: &serde_json::Value) -> SaResult<()> {
        SaTokenDao::update_object(self, key, value)
    }

    async fn get_session(&self, session_id: &str) -> SaResult<Option<SaSession>> {
        SaTokenDao::get_session(self, session_id)
    }

    async fn set_session(&self, session: &SaSession, timeout: i64) -> SaResult<()> {
        SaTokenDao::set_session(self, session, timeout)
    }

    async fn update_session(&self, session: &SaSession) -> SaResult<()> {
        SaTokenDao::update_session(self, session)
    }

    async fn search_data(
        &self,
        prefix: &str,
        keyword: &str,
        start: i64,
        size: i64,
        ascending: bool,
    ) -> SaResult<Vec<String>> {
        SaTokenDao::search_data(self, prefix, keyword, start, size, ascending)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn delegates_without_losing_errors() {
        let dao = SaTokenDaoMemory::new();
        SaTokenDao::set(&dao, "key", "value", -1).expect("set should succeed");
        assert_eq!(
            SaTokenDao::get(&dao, "key").expect("get should succeed"),
            Some("value".to_string())
        );
    }
}
