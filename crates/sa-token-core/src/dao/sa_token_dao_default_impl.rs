//! `SaTokenDaoDefaultImpl` —— 1:1 对应 Java `cn.dev33.satoken.dao.SaTokenDaoDefaultImpl`

use super::sa_token_dao::SaTokenDao;
use crate::exception::{SaResult, SaTokenException};
use crate::session::sa_session::SaSession;
use std::collections::HashMap;
use std::sync::RwLock;
use std::time::{Duration, Instant};

/// 默认内存版 DAO
pub struct SaTokenDaoDefaultImpl {
    /// 字符串数据
    pub data: RwLock<HashMap<String, (String, Option<Instant>)>>,
    /// Session 数据
    pub sessions: RwLock<HashMap<String, (SaSession, Option<Instant>)>>,
}

impl Default for SaTokenDaoDefaultImpl {
    fn default() -> Self {
        Self::new()
    }
}

impl SaTokenDaoDefaultImpl {
    pub fn new() -> Self {
        Self {
            data: RwLock::new(HashMap::new()),
            sessions: RwLock::new(HashMap::new()),
        }
    }

    fn lock_error(target: &str) -> SaTokenException {
        SaTokenException::Other {
            message: format!("内存 DAO {target} 锁已中毒"),
        }
    }

    fn is_expired(&self, key: &str) -> SaResult<bool> {
        let g = self.data.read().map_err(|_| Self::lock_error("data"))?;
        if let Some((_, exp)) = g.get(key) {
            if let Some(t) = exp {
                if Instant::now() >= *t {
                    return Ok(true);
                }
            }
            return Ok(false);
        }
        Ok(false)
    }

    fn is_session_expired(&self, key: &str) -> SaResult<bool> {
        let g = self
            .sessions
            .read()
            .map_err(|_| Self::lock_error("sessions"))?;
        if let Some((_, exp)) = g.get(key) {
            if let Some(t) = exp {
                if Instant::now() >= *t {
                    return Ok(true);
                }
            }
            return Ok(false);
        }
        Ok(false)
    }

    fn exp_for(timeout: i64) -> Option<Instant> {
        if timeout < 0 {
            None
        } else {
            Some(Instant::now() + Duration::from_secs(timeout as u64))
        }
    }
}

impl SaTokenDao for SaTokenDaoDefaultImpl {
    fn get(&self, key: &str) -> SaResult<Option<String>> {
        if self.is_expired(key)? {
            self.delete(key)?;
            return Ok(None);
        }
        Ok(self
            .data
            .read()
            .map_err(|_| Self::lock_error("data"))?
            .get(key)
            .map(|(v, _)| v.clone()))
    }
    fn set(&self, key: &str, value: &str, timeout: i64) -> SaResult<()> {
        self.data
            .write()
            .map_err(|_| Self::lock_error("data"))?
            .insert(key.to_string(), (value.to_string(), Self::exp_for(timeout)));
        Ok(())
    }
    fn update(&self, key: &str, value: &str) -> SaResult<()> {
        let mut g = self.data.write().map_err(|_| Self::lock_error("data"))?;
        if let Some(entry) = g.get_mut(key) {
            entry.0 = value.to_string();
        }
        Ok(())
    }
    fn delete(&self, key: &str) -> SaResult<()> {
        self.data
            .write()
            .map_err(|_| Self::lock_error("data"))?
            .remove(key);
        Ok(())
    }
    fn get_timeout(&self, key: &str) -> SaResult<i64> {
        let data = self.data.read().map_err(|_| Self::lock_error("data"))?;
        Ok(if let Some((_, exp)) = data.get(key) {
            if let Some(t) = exp {
                let d = t.duration_since(Instant::now());
                if d.as_secs() as i64 > 0 {
                    d.as_secs() as i64
                } else {
                    0
                }
            } else {
                -1
            }
        } else {
            -2
        })
    }
    fn update_timeout(&self, key: &str, timeout: i64) -> SaResult<()> {
        let mut g = self.data.write().map_err(|_| Self::lock_error("data"))?;
        if let Some(entry) = g.get_mut(key) {
            entry.1 = Self::exp_for(timeout);
        }
        Ok(())
    }
    fn get_object(&self, key: &str) -> SaResult<Option<serde_json::Value>> {
        self.get(key)?
            .map(|value| {
                serde_json::from_str(&value).map_err(|error| SaTokenException::JsonConvert {
                    message: error.to_string(),
                })
            })
            .transpose()
    }
    fn set_object(&self, key: &str, value: &serde_json::Value, timeout: i64) -> SaResult<()> {
        let value =
            serde_json::to_string(value).map_err(|error| SaTokenException::JsonConvert {
                message: error.to_string(),
            })?;
        self.set(key, &value, timeout)
    }
    fn update_object(&self, key: &str, value: &serde_json::Value) -> SaResult<()> {
        let value =
            serde_json::to_string(value).map_err(|error| SaTokenException::JsonConvert {
                message: error.to_string(),
            })?;
        self.update(key, &value)
    }
    fn delete_object(&self, key: &str) -> SaResult<()> {
        self.delete(key)
    }
    fn get_object_timeout(&self, key: &str) -> SaResult<i64> {
        self.get_timeout(key)
    }
    fn update_object_timeout(&self, key: &str, timeout: i64) -> SaResult<()> {
        self.update_timeout(key, timeout)
    }

    fn get_session(&self, session_id: &str) -> SaResult<Option<SaSession>> {
        if self.is_session_expired(session_id)? {
            self.delete_session(session_id)?;
            return Ok(None);
        }
        Ok(self
            .sessions
            .read()
            .map_err(|_| Self::lock_error("sessions"))?
            .get(session_id)
            .map(|(s, _)| s.clone()))
    }
    fn set_session(&self, session: &SaSession, timeout: i64) -> SaResult<()> {
        self.sessions
            .write()
            .map_err(|_| Self::lock_error("sessions"))?
            .insert(
                session.id().to_string(),
                (session.clone(), Self::exp_for(timeout)),
            );
        Ok(())
    }
    fn update_session(&self, session: &SaSession) -> SaResult<()> {
        let mut g = self
            .sessions
            .write()
            .map_err(|_| Self::lock_error("sessions"))?;
        if let Some(entry) = g.get_mut(session.id()) {
            entry.0 = session.clone();
        }
        Ok(())
    }
    fn delete_session(&self, session_id: &str) -> SaResult<()> {
        self.sessions
            .write()
            .map_err(|_| Self::lock_error("sessions"))?
            .remove(session_id);
        Ok(())
    }
    fn get_session_timeout(&self, session_id: &str) -> SaResult<i64> {
        let sessions = self
            .sessions
            .read()
            .map_err(|_| Self::lock_error("sessions"))?;
        Ok(if let Some((_, exp)) = sessions.get(session_id) {
            if let Some(t) = exp {
                let d = t.duration_since(Instant::now());
                if d.as_secs() as i64 > 0 {
                    d.as_secs() as i64
                } else {
                    0
                }
            } else {
                -1
            }
        } else {
            -2
        })
    }
    fn update_session_timeout(&self, session_id: &str, timeout: i64) -> SaResult<()> {
        let mut g = self
            .sessions
            .write()
            .map_err(|_| Self::lock_error("sessions"))?;
        if let Some(entry) = g.get_mut(session_id) {
            entry.1 = Self::exp_for(timeout);
        }
        Ok(())
    }

    fn search_data(
        &self,
        prefix: &str,
        keyword: &str,
        start: i64,
        size: i64,
        _sort_type: bool,
    ) -> SaResult<Vec<String>> {
        let g = self.data.read().map_err(|_| Self::lock_error("data"))?;
        let mut out: Vec<String> = g
            .iter()
            .filter(|(k, _)| k.starts_with(prefix) && k.contains(keyword))
            .map(|(k, _)| k.clone())
            .collect();
        // 按 key 排序
        out.sort();
        Ok(out
            .into_iter()
            .skip(start.max(0) as usize)
            .take(if size < 0 { usize::MAX } else { size as usize })
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn crud() {
        let dao = SaTokenDaoDefaultImpl::new();
        dao.set("k", "v", -1).expect("set should succeed");
        assert_eq!(
            dao.get("k").expect("get should succeed"),
            Some("v".to_string())
        );
        dao.delete("k").expect("delete should succeed");
        assert_eq!(dao.get("k").expect("get should succeed"), None);
    }

    #[test]
    fn object_round_trip() {
        let dao = SaTokenDaoDefaultImpl::new();
        dao.set_object("o", &serde_json::json!({"a": 1}), -1)
            .expect("set object should succeed");
        assert_eq!(
            dao.get_object("o").expect("get object should succeed"),
            Some(serde_json::json!({"a": 1}))
        );
    }

    #[test]
    fn search_data_returns_keys_instead_of_stored_values() {
        let dao = SaTokenDaoDefaultImpl::new();
        dao.set("satoken:var:a", "value-a", -1)
            .expect("set first search value");
        dao.set("satoken:var:b", "value-b", -1)
            .expect("set second search value");
        assert_eq!(
            dao.search_data("satoken:var:", "", 0, -1, true)
                .expect("search keys"),
            ["satoken:var:a", "satoken:var:b"]
        );
    }
}
