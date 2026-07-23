//! `SaMapPackage` —— 1:1 对应 Java `cn.dev33.satoken.dao.SaMapPackage`

use std::collections::HashMap;

/// Map 包装 trait（对应 Java `SaMapPackage<V>`）
pub trait SaMapPackage<V: Clone + Send + Sync>: Send + Sync {
    /// 获取底层被包装的源对象（Rust 侧返回只读引用计数描述）
    fn source_desc(&self) -> &'static str;

    /// 读取
    fn get(&self, key: &str) -> Option<V>;

    /// 写入
    fn put(&mut self, key: &str, value: V);

    /// 删除
    fn remove(&mut self, key: &str);

    /// 所有 key
    fn key_set(&self) -> Vec<String>;
}

/// HashMap 版 `SaMapPackage` 默认实现
pub struct SaMapPackageDefaultImpl<V> {
    pub inner: HashMap<String, V>,
}

impl<V> Default for SaMapPackageDefaultImpl<V> {
    fn default() -> Self {
        Self::new()
    }
}

impl<V> SaMapPackageDefaultImpl<V> {
    /// 创建空包装
    pub fn new() -> Self {
        Self {
            inner: HashMap::new(),
        }
    }
}

impl<V: Clone + Send + Sync> SaMapPackage<V> for SaMapPackageDefaultImpl<V> {
    fn source_desc(&self) -> &'static str {
        "HashMap"
    }

    fn get(&self, key: &str) -> Option<V> {
        self.inner.get(key).cloned()
    }

    fn put(&mut self, key: &str, value: V) {
        self.inner.insert(key.to_string(), value);
    }

    fn remove(&mut self, key: &str) {
        self.inner.remove(key);
    }

    fn key_set(&self) -> Vec<String> {
        self.inner.keys().cloned().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn map_package_crud() {
        let mut pkg = SaMapPackageDefaultImpl::<String>::new();
        pkg.put("k", "v".to_string());
        assert_eq!(pkg.get("k"), Some("v".to_string()));
        assert!(pkg.key_set().contains(&"k".to_string()));
        pkg.remove("k");
        assert_eq!(pkg.get("k"), None);
    }
}
