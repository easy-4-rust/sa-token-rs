//! Request-scoped storage adapter (`SaStorage`).

use std::any::Any;
use std::collections::HashMap;
use std::sync::RwLock;

use sa_token_core::context::model::sa_storage::SaStorage;

/// Axum 存储适配（请求级临时存储）
pub struct AxumStorage {
    /// 数据
    data: RwLock<HashMap<String, String>>,
}

impl Default for AxumStorage {
    fn default() -> Self {
        Self {
            data: RwLock::new(HashMap::new()),
        }
    }
}

impl AxumStorage {
    /// Clears all request-scoped entries.
    pub fn clear(&self) {
        self.data.write().unwrap().clear();
    }
}

impl SaStorage for AxumStorage {
    fn source(&self) -> &dyn Any {
        self
    }

    fn get(&self, key: &str) -> Option<String> {
        self.data.read().unwrap().get(key).cloned()
    }

    fn set(&self, key: &str, value: &str) {
        self.data
            .write()
            .unwrap()
            .insert(key.to_string(), value.to_string());
    }

    fn delete(&self, key: &str) {
        self.data.write().unwrap().remove(key);
    }
}
