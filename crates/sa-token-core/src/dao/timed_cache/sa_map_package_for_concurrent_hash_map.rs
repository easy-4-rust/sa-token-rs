//! `SaMapPackageForConcurrentHashMap` —— 1:1 对应 Java `cn.dev33.satoken.dao.SaMapPackageForConcurrentHashMap`

use super::sa_map_package::SaMapPackage;
use std::collections::HashMap;
use std::sync::RwLock;

/// 多线程版 `SaMapPackage`（通过 `RwLock` 包装 HashMap）
pub struct SaMapPackageForConcurrentHashMap<V> {
    inner: RwLock<HashMap<String, V>>,
}

impl<V> Default for SaMapPackageForConcurrentHashMap<V> {
    fn default() -> Self {
        Self::new()
    }
}

impl<V> SaMapPackageForConcurrentHashMap<V> {
    /// 创建空并发 Map 包装
    pub fn new() -> Self {
        Self {
            inner: RwLock::new(HashMap::new()),
        }
    }
}

impl<V: Clone + Send + Sync> SaMapPackage<V> for SaMapPackageForConcurrentHashMap<V> {
    fn source_desc(&self) -> &'static str {
        "RwLock<HashMap>"
    }

    fn get(&self, key: &str) -> Option<V> {
        self.inner.read().ok()?.get(key).cloned()
    }

    fn put(&mut self, key: &str, value: V) {
        if let Ok(mut guard) = self.inner.write() {
            guard.insert(key.to_string(), value);
        }
    }

    fn remove(&mut self, key: &str) {
        if let Ok(mut guard) = self.inner.write() {
            guard.remove(key);
        }
    }

    fn key_set(&self) -> Vec<String> {
        self.inner
            .read()
            .map(|guard| guard.keys().cloned().collect())
            .unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn concurrent_map_package_crud() {
        let mut pkg = SaMapPackageForConcurrentHashMap::<String>::new();
        pkg.put("k", "v".to_string());
        assert_eq!(pkg.get("k"), Some("v".to_string()));
        pkg.remove("k");
        assert_eq!(pkg.get("k"), None);
    }
}
