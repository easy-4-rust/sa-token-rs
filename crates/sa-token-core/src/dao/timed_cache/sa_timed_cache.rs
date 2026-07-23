//! `SaTimedCache` —— 1:1 对应 Java `cn.dev33.satoken.dao.SaTimedCache`
//!
//! 定时缓存：惰性过期检查 + 可选手动刷新。

use super::sa_map_package::{SaMapPackage, SaMapPackageDefaultImpl};
use crate::dao::sa_token_dao::{NEVER_EXPIRE, NOT_VALUE_EXPIRE};

/// 带过期时间的本地缓存（对应 Java `SaTimedCache`）
pub struct SaTimedCache {
    /// 数据集合
    pub data_map: SaMapPackageDefaultImpl<String>,
    /// 过期时间集合（毫秒时间戳；`NEVER_EXPIRE` 表示永不过期）
    pub expire_map: SaMapPackageDefaultImpl<i64>,
}

impl Default for SaTimedCache {
    fn default() -> Self {
        Self::new()
    }
}

impl SaTimedCache {
    /// 创建空缓存
    pub fn new() -> Self {
        Self {
            data_map: SaMapPackageDefaultImpl::new(),
            expire_map: SaMapPackageDefaultImpl::new(),
        }
    }

    /// 使用自定义 Map 包装创建
    pub fn with_maps(
        data_map: SaMapPackageDefaultImpl<String>,
        expire_map: SaMapPackageDefaultImpl<i64>,
    ) -> Self {
        Self { data_map, expire_map }
    }

    /// 获取 Object，过期则清除
    pub fn get_object(&mut self, key: &str) -> Option<String> {
        self.clear_key_by_timeout(key);
        self.data_map.get(key)
    }

    /// 写入 Object 并设定存活时间（秒）
    pub fn set_object(&mut self, key: &str, object: &str, timeout: i64) {
        if timeout == 0 || timeout <= NOT_VALUE_EXPIRE {
            return;
        }
        self.data_map.put(key, object.to_string());
        let expire = if timeout == NEVER_EXPIRE {
            NEVER_EXPIRE
        } else {
            now_millis() + timeout * 1000
        };
        self.expire_map.put(key, expire);
    }

    /// 更新 Object（过期时间不变）
    pub fn update_object(&mut self, key: &str, object: &str) {
        if self.get_key_timeout(key) == NOT_VALUE_EXPIRE {
            return;
        }
        self.data_map.put(key, object.to_string());
    }

    /// 删除 Object
    pub fn delete_object(&mut self, key: &str) {
        self.data_map.remove(key);
        self.expire_map.remove(key);
    }

    /// 获取 Object 剩余存活时间（秒）
    pub fn get_object_timeout(&mut self, key: &str) -> i64 {
        self.get_key_timeout(key)
    }

    /// 修改 Object 剩余存活时间（秒）
    pub fn update_object_timeout(&mut self, key: &str, timeout: i64) {
        let expire = if timeout == NEVER_EXPIRE {
            NEVER_EXPIRE
        } else {
            now_millis() + timeout * 1000
        };
        self.expire_map.put(key, expire);
    }

    /// 所有 key
    pub fn key_set(&self) -> Vec<String> {
        self.data_map.key_set()
    }

    /// 清理所有已过期 key
    pub fn refresh_data_map(&mut self) {
        let keys = self.expire_map.key_set();
        for key in keys {
            self.clear_key_by_timeout(&key);
        }
    }

    /// 若 key 已过期则立即清除
    fn clear_key_by_timeout(&mut self, key: &str) {
        let Some(expiration) = self.expire_map.get(key) else {
            return;
        };
        if expiration != NEVER_EXPIRE && expiration < now_millis() {
            self.data_map.remove(key);
            self.expire_map.remove(key);
        }
    }

    /// 获取 key 剩余存活时间（秒）
    fn get_key_timeout(&mut self, key: &str) -> i64 {
        self.clear_key_by_timeout(key);
        let Some(expire) = self.expire_map.get(key) else {
            return NOT_VALUE_EXPIRE;
        };
        if expire == NEVER_EXPIRE {
            return NEVER_EXPIRE;
        }
        let timeout = (expire - now_millis()) / 1000;
        if timeout < 0 {
            self.data_map.remove(key);
            self.expire_map.remove(key);
            return NOT_VALUE_EXPIRE;
        }
        timeout
    }
}

fn now_millis() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn crud_and_never_expire() {
        let mut cache = SaTimedCache::new();
        cache.set_object("k", "v", NEVER_EXPIRE);
        assert_eq!(cache.get_object("k"), Some("v".to_string()));
        assert_eq!(cache.get_object_timeout("k"), NEVER_EXPIRE);
        cache.delete_object("k");
        assert_eq!(cache.get_object("k"), None);
    }

    #[test]
    fn expired_key_removed() {
        let mut cache = SaTimedCache::new();
        cache.set_object("k", "v", 1);
        // 手动写入已过期时间戳
        cache.expire_map.put("k", now_millis() - 1000);
        assert_eq!(cache.get_object("k"), None);
    }
}
