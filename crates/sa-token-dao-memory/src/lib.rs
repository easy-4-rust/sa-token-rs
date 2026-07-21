//! Memory DAO 实现（对应 Java `SaTokenDaoDefaultImpl`）。
//!
//! 基于内存的持久化实现，适用于单机和测试场景。

use std::collections::HashMap;
use std::sync::RwLock;
use std::time::{SystemTime, UNIX_EPOCH};

use sa_token_core::dao::sa_token_dao::SaTokenDao;
use sa_token_core::session::sa_session::SaSession;

/// 内存条目
#[derive(Debug, Clone)]
struct MemoryEntry {
    /// 值
    value: String,
    /// 过期时间（秒级时间戳），-1 表示永不过期
    expire_time: i64,
}

/// 内存条目（Object 类型）
#[derive(Debug, Clone)]
struct ObjectEntry {
    /// 值
    value: serde_json::Value,
    /// 过期时间（秒级时间戳），-1 表示永不过期
    expire_time: i64,
}

/// 内存条目（Session 类型）
#[derive(Debug, Clone)]
struct SessionEntry {
    /// 值
    value: SaSession,
    /// 过期时间（秒级时间戳），-1 表示永不过期
    expire_time: i64,
}

/// 内存 DAO 实现
pub struct SaTokenDaoMemory {
    /// 字符串数据
    data: RwLock<HashMap<String, MemoryEntry>>,
    /// Object 数据
    object_data: RwLock<HashMap<String, ObjectEntry>>,
    /// Session 数据
    session_data: RwLock<HashMap<String, SessionEntry>>,
}

impl Default for SaTokenDaoMemory {
    fn default() -> Self {
        Self::new()
    }
}

impl SaTokenDaoMemory {
    /// 创建新的 Memory DAO
    pub fn new() -> Self {
        Self {
            data: RwLock::new(HashMap::new()),
            object_data: RwLock::new(HashMap::new()),
            session_data: RwLock::new(HashMap::new()),
        }
    }

    /// 获取当前时间戳（秒）
    fn now_timestamp() -> i64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64
    }

    /// 计算过期时间
    fn calc_expire_time(timeout: i64) -> i64 {
        if timeout <= 0 {
            -1
        } else {
            Self::now_timestamp() + timeout
        }
    }

    /// 检查是否过期
    fn is_expired(expire_time: i64) -> bool {
        if expire_time < 0 {
            return false;
        }
        Self::now_timestamp() > expire_time
    }

    /// 清理过期数据
    fn cleanup_expired(&self) {
        let now = Self::now_timestamp();

        // 清理字符串数据
        if let Ok(mut data) = self.data.write() {
            data.retain(|_, entry| entry.expire_time < 0 || entry.expire_time > now);
        }

        // 清理 Object 数据
        if let Ok(mut data) = self.object_data.write() {
            data.retain(|_, entry| entry.expire_time < 0 || entry.expire_time > now);
        }

        // 清理 Session 数据
        if let Ok(mut data) = self.session_data.write() {
            data.retain(|_, entry| entry.expire_time < 0 || entry.expire_time > now);
        }
    }
}

impl SaTokenDao for SaTokenDaoMemory {
    fn get(&self, key: &str) -> Option<String> {
        let data = self.data.read().ok()?;
        let entry = data.get(key)?;
        if Self::is_expired(entry.expire_time) {
            return None;
        }
        Some(entry.value.clone())
    }

    fn set(&self, key: &str, value: &str, timeout: i64) {
        if let Ok(mut data) = self.data.write() {
            data.insert(
                key.to_string(),
                MemoryEntry {
                    value: value.to_string(),
                    expire_time: Self::calc_expire_time(timeout),
                },
            );
        }
    }

    fn update(&self, key: &str, value: &str) {
        if let Ok(mut data) = self.data.write() {
            if let Some(entry) = data.get_mut(key) {
                entry.value = value.to_string();
            }
        }
    }

    fn delete(&self, key: &str) {
        if let Ok(mut data) = self.data.write() {
            data.remove(key);
        }
    }

    fn get_timeout(&self, key: &str) -> i64 {
        let data = match self.data.read() {
            Ok(d) => d,
            Err(_) => return -2,
        };
        let entry = match data.get(key) {
            Some(e) => e,
            None => return -2,
        };
        if Self::is_expired(entry.expire_time) {
            return -2;
        }
        if entry.expire_time < 0 {
            -1
        } else {
            entry.expire_time - Self::now_timestamp()
        }
    }

    fn update_timeout(&self, key: &str, timeout: i64) {
        if let Ok(mut data) = self.data.write() {
            if let Some(entry) = data.get_mut(key) {
                entry.expire_time = Self::calc_expire_time(timeout);
            }
        }
    }

    fn get_object(&self, key: &str) -> Option<serde_json::Value> {
        let data = self.object_data.read().ok()?;
        let entry = data.get(key)?;
        if Self::is_expired(entry.expire_time) {
            return None;
        }
        Some(entry.value.clone())
    }

    fn set_object(&self, key: &str, value: &serde_json::Value, timeout: i64) {
        if let Ok(mut data) = self.object_data.write() {
            data.insert(
                key.to_string(),
                ObjectEntry {
                    value: value.clone(),
                    expire_time: Self::calc_expire_time(timeout),
                },
            );
        }
    }

    fn update_object(&self, key: &str, value: &serde_json::Value) {
        if let Ok(mut data) = self.object_data.write() {
            if let Some(entry) = data.get_mut(key) {
                entry.value = value.clone();
            }
        }
    }

    fn delete_object(&self, key: &str) {
        if let Ok(mut data) = self.object_data.write() {
            data.remove(key);
        }
    }

    fn get_object_timeout(&self, key: &str) -> i64 {
        let data = match self.object_data.read() {
            Ok(d) => d,
            Err(_) => return -2,
        };
        let entry = match data.get(key) {
            Some(e) => e,
            None => return -2,
        };
        if Self::is_expired(entry.expire_time) {
            return -2;
        }
        if entry.expire_time < 0 {
            -1
        } else {
            entry.expire_time - Self::now_timestamp()
        }
    }

    fn update_object_timeout(&self, key: &str, timeout: i64) {
        if let Ok(mut data) = self.object_data.write() {
            if let Some(entry) = data.get_mut(key) {
                entry.expire_time = Self::calc_expire_time(timeout);
            }
        }
    }

    fn get_session(&self, session_id: &str) -> Option<SaSession> {
        let data = self.session_data.read().ok()?;
        let entry = data.get(session_id)?;
        if Self::is_expired(entry.expire_time) {
            return None;
        }
        Some(entry.value.clone())
    }

    fn set_session(&self, session: &SaSession, timeout: i64) {
        if let Ok(mut data) = self.session_data.write() {
            data.insert(
                session.id().to_string(),
                SessionEntry {
                    value: session.clone(),
                    expire_time: Self::calc_expire_time(timeout),
                },
            );
        }
    }

    fn update_session(&self, session: &SaSession) {
        if let Ok(mut data) = self.session_data.write() {
            if let Some(entry) = data.get_mut(session.id()) {
                entry.value = session.clone();
            }
        }
    }

    fn delete_session(&self, session_id: &str) {
        if let Ok(mut data) = self.session_data.write() {
            data.remove(session_id);
        }
    }

    fn get_session_timeout(&self, session_id: &str) -> i64 {
        let data = match self.session_data.read() {
            Ok(d) => d,
            Err(_) => return -2,
        };
        let entry = match data.get(session_id) {
            Some(e) => e,
            None => return -2,
        };
        if Self::is_expired(entry.expire_time) {
            return -2;
        }
        if entry.expire_time < 0 {
            -1
        } else {
            entry.expire_time - Self::now_timestamp()
        }
    }

    fn update_session_timeout(&self, session_id: &str, timeout: i64) {
        if let Ok(mut data) = self.session_data.write() {
            if let Some(entry) = data.get_mut(session_id) {
                entry.expire_time = Self::calc_expire_time(timeout);
            }
        }
    }

    fn search_data(
        &self,
        prefix: &str,
        keyword: &str,
        start: i64,
        size: i64,
        sort_type: bool,
    ) -> Vec<String> {
        let data = match self.data.read() {
            Ok(d) => d,
            Err(_) => return Vec::new(),
        };

        let now = Self::now_timestamp();
        let mut results: Vec<String> = data
            .iter()
            .filter(|(key, entry)| {
                key.starts_with(prefix)
                    && (keyword.is_empty() || key.contains(keyword))
                    && (entry.expire_time < 0 || entry.expire_time > now)
            })
            .map(|(key, _)| key.clone())
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
