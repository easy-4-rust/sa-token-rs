//! `SaStorageForMock` —— 1:1 对应 Java `cn.dev33.satoken.context.model.SaStorageForMock`

use super::super::model::sa_storage::SaStorage;
use std::collections::HashMap;
use std::sync::RwLock;

/// 简单的 KV 存储（线程安全）。
pub struct SaStorageForMock {
    inner: RwLock<HashMap<String, String>>,
}

impl SaStorageForMock {
    pub fn new() -> Self {
        Self {
            inner: RwLock::new(HashMap::new()),
        }
    }

    pub fn put(&self, key: impl Into<String>, value: impl Into<String>) {
        self.inner.write().unwrap().insert(key.into(), value.into());
    }

    pub fn take(&self, key: &str) -> Option<String> {
        self.inner.read().unwrap().get(key).cloned()
    }

    pub fn remove(&self, key: &str) {
        self.inner.write().unwrap().remove(key);
    }

    pub fn clear(&self) {
        self.inner.write().unwrap().clear();
    }
}

impl Default for SaStorageForMock {
    fn default() -> Self {
        Self::new()
    }
}

impl SaStorage for SaStorageForMock {
    fn source(&self) -> &dyn std::any::Any {
        self
    }
    fn get(&self, key: &str) -> Option<String> {
        self.take(key)
    }
    fn set(&self, key: &str, value: &str) {
        self.put(key, value)
    }
    fn delete(&self, key: &str) {
        self.remove(key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn crud() {
        let s = SaStorageForMock::new();
        s.put("k", "v");
        assert_eq!(s.take("k"), Some("v".to_string()));
        s.remove("k");
        assert_eq!(s.take("k"), None);
    }

    #[test]
    fn trait_interface() {
        let s = SaStorageForMock::new();
        s.set("a", "1");
        s.set("b", "2");
        assert_eq!(s.get("a"), Some("1".to_string()));
        assert_eq!(s.get("b"), Some("2".to_string()));
        s.delete("a");
        assert_eq!(s.get("a"), None);
    }
}
