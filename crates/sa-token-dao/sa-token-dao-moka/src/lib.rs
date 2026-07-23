//! `sa-token-dao-moka` —— moka 后端的内存 DAO 适配器。
//!
//! 对应 Java Sa-Token 的 `Caffeine` 内存缓存后端。moka 是 Rust 生态中
//! 性能与 Caffeine 最接近的进程内缓存实现（基于 LFU/LRU + 时间窗口的 TinyLFU
//! 近似算法），天然支持 TTL、最大容量、按 key 失效。
//!
//! # 关键设计
//! - 1:1 复用 `sa_token_core::dao::sa_token_dao::SaTokenDao` trait
//! - moka 的 `Cache<String, (Value, Expiry)>` 模式手动管理过期时间，
//!   因为 moka 的 entry-level expire 是绝对时间戳而非相对时长
//! - 提供与 `SaTokenDaoMemory` 完全相同的语义，但支持 `max_capacity` 容量限制
//!   与可配置的淘汰策略

use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use moka::future::Cache;
use moka::policy::EvictionPolicy;
use sa_token_core::dao::async_sa_token_dao::AsyncSaTokenDao;
use sa_token_core::dao::sa_token_dao::SaTokenDao;
use sa_token_core::exception::{SaResult, SaTokenException};
use sa_token_core::session::sa_session::SaSession;

/// 内部存储的 entry —— (value, expire_at_millis)
#[derive(Clone)]
struct Entry {
    value: String,
    /// 绝对到期时间（毫秒）。`None` 表示永不过期。
    expire_at_ms: Option<i64>,
}

impl Entry {
    fn expired(&self) -> bool {
        match self.expire_at_ms {
            None => false,
            Some(deadline) => now_millis() >= deadline,
        }
    }
}

fn now_millis() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(i64::MAX)
}

fn timeout_to_deadline_ms(timeout: i64) -> Option<i64> {
    if timeout < 0 {
        None
    } else {
        Some(now_millis() + timeout * 1000)
    }
}

/// 内部 Object entry —— (json, expire_at_millis)
#[derive(Clone)]
struct ObjectEntry {
    value: serde_json::Value,
    expire_at_ms: Option<i64>,
}

impl ObjectEntry {
    fn expired(&self) -> bool {
        match self.expire_at_ms {
            None => false,
            Some(deadline) => now_millis() >= deadline,
        }
    }
}

/// 内部 Session entry
#[derive(Clone)]
struct SessionEntry {
    session: SaSession,
    expire_at_ms: Option<i64>,
}

impl SessionEntry {
    fn expired(&self) -> bool {
        match self.expire_at_ms {
            None => false,
            Some(deadline) => now_millis() >= deadline,
        }
    }
}

/// moka 后端的 sa-token DAO 适配器
///
/// 提供与 `SaTokenDaoMemory` 相同的语义，但使用 moka 作为底层缓存，
/// 支持容量上限、TTL 自动失效。
#[derive(Clone)]
pub struct SaTokenDaoMoka {
    /// 字符串存储
    kv: Cache<String, Arc<Entry>>,
    /// JSON 对象存储
    objs: Cache<String, Arc<ObjectEntry>>,
    /// Session 存储
    sessions: Cache<String, Arc<SessionEntry>>,
}

impl Default for SaTokenDaoMoka {
    fn default() -> Self {
        Self::builder().build()
    }
}

impl SaTokenDaoMoka {
    /// 创建一个 builder 以精细配置容量与淘汰策略
    pub fn builder() -> SaTokenDaoMokaBuilder {
        SaTokenDaoMokaBuilder::default()
    }

    /// 强制让 moka 触发一次失效条目清理（在测试中可立即观察到过期效果）
    pub fn maintenance(&self) {
        let kv = self.kv.clone();
        let objs = self.objs.clone();
        let sessions = self.sessions.clone();
        run_on_runtime(move || async move {
            kv.run_pending_tasks().await;
            objs.run_pending_tasks().await;
            sessions.run_pending_tasks().await;
        })
        .ok();
    }

    /// 估算当前 kv 缓存条目数（含可能过期的，等价于 moka 的 entry_count）
    pub fn entry_count(&self) -> u64 {
        self.kv.entry_count()
    }

    /// 主动 invalidate 某个 key
    pub fn invalidate(&self, key: &str) {
        let kv = self.kv.clone();
        let key = key.to_string();
        run_on_runtime(move || async move {
            kv.invalidate(&key).await;
        })
        .ok();
    }
}

/// 配置 builder
#[derive(Default)]
pub struct SaTokenDaoMokaBuilder {
    max_capacity: Option<u64>,
}

impl SaTokenDaoMokaBuilder {
    /// 设置最大容量（条目数）。默认无限。
    pub fn max_capacity(mut self, cap: u64) -> Self {
        self.max_capacity = Some(cap);
        self
    }

    /// 构建 moka DAO 实例
    pub fn build(self) -> SaTokenDaoMoka {
        let mut kv_builder = Cache::builder().eviction_policy(EvictionPolicy::tiny_lfu());
        let mut obj_builder = Cache::builder().eviction_policy(EvictionPolicy::tiny_lfu());
        let mut sess_builder = Cache::builder().eviction_policy(EvictionPolicy::tiny_lfu());
        if let Some(cap) = self.max_capacity {
            kv_builder = kv_builder.max_capacity(cap);
            obj_builder = obj_builder.max_capacity(cap);
            sess_builder = sess_builder.max_capacity(cap);
        }
        SaTokenDaoMoka {
            kv: kv_builder.build(),
            objs: obj_builder.build(),
            sessions: sess_builder.build(),
        }
    }
}

/// 工具：把 moka 没有的 "按相对 TTL 失效" 逻辑用 entry-level expire_at
/// 模拟；查询前先检查 `expired()`，过期则 invalidate。
async fn touch_entry(cache: &Cache<String, Arc<Entry>>, key: &str) -> Option<Arc<Entry>> {
    if let Some(entry) = cache.get(key).await {
        if entry.expired() {
            cache.invalidate(key).await;
            return None;
        }
        return Some(entry);
    }
    None
}

async fn touch_object(
    cache: &Cache<String, Arc<ObjectEntry>>,
    key: &str,
) -> Option<Arc<ObjectEntry>> {
    if let Some(entry) = cache.get(key).await {
        if entry.expired() {
            cache.invalidate(key).await;
            return None;
        }
        return Some(entry);
    }
    None
}

async fn touch_session(
    cache: &Cache<String, Arc<SessionEntry>>,
    key: &str,
) -> Option<Arc<SessionEntry>> {
    if let Some(entry) = cache.get(key).await {
        if entry.expired() {
            cache.invalidate(key).await;
            return None;
        }
        return Some(entry);
    }
    None
}

impl SaTokenDao for SaTokenDaoMoka {
    fn get(&self, key: &str) -> SaResult<Option<String>> {
        let kv = self.kv.clone();
        let key = key.to_string();
        let handle = tokio::runtime::Handle::try_current().map_err(|_| {
            SaTokenException::other("sa-token-dao-moka requires an active tokio runtime")
        })?;
        let entry = tokio::task::block_in_place(|| {
            handle.block_on(async move { touch_entry(&kv, &key).await })
        });
        Ok(entry.map(|e| e.value.clone()))
    }

    fn set(&self, key: &str, value: &str, timeout: i64) -> SaResult<()> {
        let kv = self.kv.clone();
        let key = key.to_string();
        let value = value.to_string();
        let deadline = timeout_to_deadline_ms(timeout);
        let entry = Arc::new(Entry {
            value,
            expire_at_ms: deadline,
        });
        run_on_runtime(move || async move {
            kv.insert(key, entry).await;
        })
    }

    fn update(&self, key: &str, value: &str) -> SaResult<()> {
        let kv = self.kv.clone();
        let key = key.to_string();
        let value = value.to_string();
        run_on_runtime(move || async move {
            if let Some(existing) = kv.get(&key).await {
                let new_entry = Arc::new(Entry {
                    value,
                    expire_at_ms: existing.expire_at_ms,
                });
                kv.insert(key, new_entry).await;
            } else {
                kv.insert(
                    key,
                    Arc::new(Entry {
                        value,
                        expire_at_ms: None,
                    }),
                )
                .await;
            }
        })
    }

    fn delete(&self, key: &str) -> SaResult<()> {
        let kv = self.kv.clone();
        let key = key.to_string();
        run_on_runtime(move || async move {
            kv.invalidate(&key).await;
        })
    }

    fn get_timeout(&self, key: &str) -> SaResult<i64> {
        let kv = self.kv.clone();
        let key = key.to_string();
        let handle = tokio::runtime::Handle::try_current().map_err(|_| {
            SaTokenException::other("sa-token-dao-moka requires an active tokio runtime")
        })?;
        let entry = tokio::task::block_in_place(|| {
            handle.block_on(async move { touch_entry(&kv, &key).await })
        });
        match entry.and_then(|e| e.expire_at_ms) {
            None => Ok(-1),
            Some(deadline) => Ok(((deadline - now_millis()) / 1000).max(0)),
        }
    }

    fn update_timeout(&self, key: &str, timeout: i64) -> SaResult<()> {
        let kv = self.kv.clone();
        let key = key.to_string();
        let new_deadline = timeout_to_deadline_ms(timeout);
        run_on_runtime(move || async move {
            if let Some(existing) = kv.get(&key).await {
                let new_entry = Arc::new(Entry {
                    value: existing.value.clone(),
                    expire_at_ms: new_deadline,
                });
                kv.insert(key, new_entry).await;
            }
        })
    }

    fn get_object(&self, key: &str) -> SaResult<Option<serde_json::Value>> {
        let objs = self.objs.clone();
        let key = key.to_string();
        let handle = tokio::runtime::Handle::try_current().map_err(|_| {
            SaTokenException::other("sa-token-dao-moka requires an active tokio runtime")
        })?;
        let entry = tokio::task::block_in_place(|| {
            handle.block_on(async move { touch_object(&objs, &key).await })
        });
        Ok(entry.map(|e| e.value.clone()))
    }

    fn set_object(&self, key: &str, value: &serde_json::Value, timeout: i64) -> SaResult<()> {
        let objs = self.objs.clone();
        let key = key.to_string();
        let value = value.clone();
        let deadline = timeout_to_deadline_ms(timeout);
        let entry = Arc::new(ObjectEntry {
            value,
            expire_at_ms: deadline,
        });
        run_on_runtime(move || async move {
            objs.insert(key, entry).await;
        })
    }

    fn update_object(&self, key: &str, value: &serde_json::Value) -> SaResult<()> {
        let objs = self.objs.clone();
        let key = key.to_string();
        let value = value.clone();
        run_on_runtime(move || async move {
            if let Some(existing) = objs.get(&key).await {
                let new_entry = Arc::new(ObjectEntry {
                    value,
                    expire_at_ms: existing.expire_at_ms,
                });
                objs.insert(key, new_entry).await;
            } else {
                objs.insert(
                    key,
                    Arc::new(ObjectEntry {
                        value,
                        expire_at_ms: None,
                    }),
                )
                .await;
            }
        })
    }

    fn delete_object(&self, key: &str) -> SaResult<()> {
        let objs = self.objs.clone();
        let key = key.to_string();
        run_on_runtime(move || async move {
            objs.invalidate(&key).await;
        })
    }

    fn get_object_timeout(&self, key: &str) -> SaResult<i64> {
        let objs = self.objs.clone();
        let key = key.to_string();
        let handle = tokio::runtime::Handle::try_current().map_err(|_| {
            SaTokenException::other("sa-token-dao-moka requires an active tokio runtime")
        })?;
        let entry = tokio::task::block_in_place(|| {
            handle.block_on(async move { touch_object(&objs, &key).await })
        });
        match entry.and_then(|e| e.expire_at_ms) {
            None => Ok(-1),
            Some(deadline) => Ok(((deadline - now_millis()) / 1000).max(0)),
        }
    }

    fn update_object_timeout(&self, key: &str, timeout: i64) -> SaResult<()> {
        let objs = self.objs.clone();
        let key = key.to_string();
        let new_deadline = timeout_to_deadline_ms(timeout);
        run_on_runtime(move || async move {
            if let Some(existing) = objs.get(&key).await {
                let new_entry = Arc::new(ObjectEntry {
                    value: existing.value.clone(),
                    expire_at_ms: new_deadline,
                });
                objs.insert(key, new_entry).await;
            }
        })
    }

    fn get_session(&self, session_id: &str) -> SaResult<Option<SaSession>> {
        let sessions = self.sessions.clone();
        let key = session_id.to_string();
        let handle = tokio::runtime::Handle::try_current().map_err(|_| {
            SaTokenException::other("sa-token-dao-moka requires an active tokio runtime")
        })?;
        let entry = tokio::task::block_in_place(|| {
            handle.block_on(async move { touch_session(&sessions, &key).await })
        });
        Ok(entry.map(|e| e.session.clone()))
    }

    fn set_session(&self, session: &SaSession, timeout: i64) -> SaResult<()> {
        let sessions = self.sessions.clone();
        let key = session.id().to_string();
        let session = session.clone();
        let deadline = timeout_to_deadline_ms(timeout);
        let entry = Arc::new(SessionEntry {
            session,
            expire_at_ms: deadline,
        });
        run_on_runtime(move || async move {
            sessions.insert(key, entry).await;
        })
    }

    fn update_session(&self, session: &SaSession) -> SaResult<()> {
        let sessions = self.sessions.clone();
        let key = session.id().to_string();
        let session = session.clone();
        run_on_runtime(move || async move {
            if let Some(existing) = sessions.get(&key).await {
                let new_entry = Arc::new(SessionEntry {
                    session,
                    expire_at_ms: existing.expire_at_ms,
                });
                sessions.insert(key, new_entry).await;
            } else {
                sessions.insert(
                    key,
                    Arc::new(SessionEntry {
                        session,
                        expire_at_ms: None,
                    }),
                )
                .await;
            }
        })
    }

    fn delete_session(&self, session_id: &str) -> SaResult<()> {
        let sessions = self.sessions.clone();
        let key = session_id.to_string();
        run_on_runtime(move || async move {
            sessions.invalidate(&key).await;
        })
    }

    fn get_session_timeout(&self, session_id: &str) -> SaResult<i64> {
        let sessions = self.sessions.clone();
        let key = session_id.to_string();
        let handle = tokio::runtime::Handle::try_current().map_err(|_| {
            SaTokenException::other("sa-token-dao-moka requires an active tokio runtime")
        })?;
        let entry = tokio::task::block_in_place(|| {
            handle.block_on(async move { touch_session(&sessions, &key).await })
        });
        match entry.and_then(|e| e.expire_at_ms) {
            None => Ok(-1),
            Some(deadline) => Ok(((deadline - now_millis()) / 1000).max(0)),
        }
    }

    fn update_session_timeout(&self, session_id: &str, timeout: i64) -> SaResult<()> {
        let sessions = self.sessions.clone();
        let key = session_id.to_string();
        let new_deadline = timeout_to_deadline_ms(timeout);
        run_on_runtime(move || async move {
            if let Some(existing) = sessions.get(&key).await {
                let new_entry = Arc::new(SessionEntry {
                    session: existing.session.clone(),
                    expire_at_ms: new_deadline,
                });
                sessions.insert(key, new_entry).await;
            }
        })
    }

    fn search_data(
        &self,
        prefix: &str,
        keyword: &str,
        start: i64,
        size: i64,
        sort_type: bool,
    ) -> SaResult<Vec<String>> {
        // moka 不直接支持 key 枚举，但我们可以用一个并发的 Vec 索引来模拟。
        // 为了保持简单且不影响主流程（生产环境主要用 redis），我们用 tokio 桥接做一次扫描。
        let kv = self.kv.clone();
        let objs = self.objs.clone();
        let sessions = self.sessions.clone();
        let prefix = prefix.to_string();
        let keyword = keyword.to_string();
        let handle = tokio::runtime::Handle::try_current().map_err(|_| {
            SaTokenException::other("sa-token-dao-moka requires an active tokio runtime")
        })?;
        let mut keys = tokio::task::block_in_place(|| {
            handle.block_on(async move {
                let mut result: Vec<String> = Vec::new();
                for (k, _v) in kv.iter() {
                    let key = k.as_ref();
                    if key.starts_with(&prefix)
                        && (keyword.is_empty() || key.contains(&keyword))
                    {
                        result.push(key.to_string());
                    }
                }
                for (k, _v) in objs.iter() {
                    let key = k.as_ref();
                    if key.starts_with(&prefix)
                        && (keyword.is_empty() || key.contains(&keyword))
                    {
                        result.push(key.to_string());
                    }
                }
                for (k, _v) in sessions.iter() {
                    let key = k.as_ref();
                    if key.starts_with(&prefix)
                        && (keyword.is_empty() || key.contains(&keyword))
                    {
                        result.push(key.to_string());
                    }
                }
                result
            })
        });
        if sort_type {
            keys.sort();
        } else {
            keys.sort_by(|a, b| b.cmp(a));
        }
        let start_idx = start.max(0) as usize;
        let end_idx = if size < 0 {
            keys.len()
        } else {
            (start_idx + size as usize).min(keys.len())
        };
        Ok(keys
            .into_iter()
            .skip(start_idx)
            .take(end_idx - start_idx)
            .collect())
    }
}

#[async_trait]
impl AsyncSaTokenDao for SaTokenDaoMoka {
    async fn get(&self, key: &str) -> SaResult<Option<String>> {
        Ok(touch_entry(&self.kv, key).await.map(|e| e.value.clone()))
    }

    async fn set(&self, key: &str, value: &str, timeout: i64) -> SaResult<()> {
        let entry = Arc::new(Entry {
            value: value.to_string(),
            expire_at_ms: timeout_to_deadline_ms(timeout),
        });
        self.kv.insert(key.to_string(), entry).await;
        Ok(())
    }

    async fn update(&self, key: &str, value: &str) -> SaResult<()> {
        if let Some(existing) = self.kv.get(key).await {
            self.kv
                .insert(
                    key.to_string(),
                    Arc::new(Entry {
                        value: value.to_string(),
                        expire_at_ms: existing.expire_at_ms,
                    }),
                )
                .await;
        } else {
            self.kv
                .insert(
                    key.to_string(),
                    Arc::new(Entry {
                        value: value.to_string(),
                        expire_at_ms: None,
                    }),
                )
                .await;
        }
        Ok(())
    }

    async fn delete(&self, key: &str) -> SaResult<()> {
        self.kv.invalidate(key).await;
        Ok(())
    }

    async fn get_timeout(&self, key: &str) -> SaResult<i64> {
        match touch_entry(&self.kv, key).await {
            Some(e) => match e.expire_at_ms {
                None => Ok(-1),
                Some(d) => Ok(((d - now_millis()) / 1000).max(0)),
            },
            None => Ok(-2),
        }
    }

    async fn update_timeout(&self, key: &str, timeout: i64) -> SaResult<()> {
        if let Some(existing) = self.kv.get(key).await {
            self.kv
                .insert(
                    key.to_string(),
                    Arc::new(Entry {
                        value: existing.value.clone(),
                        expire_at_ms: timeout_to_deadline_ms(timeout),
                    }),
                )
                .await;
        }
        Ok(())
    }

    async fn get_object(&self, key: &str) -> SaResult<Option<serde_json::Value>> {
        Ok(touch_object(&self.objs, key).await.map(|e| e.value.clone()))
    }

    async fn set_object(&self, key: &str, value: &serde_json::Value, timeout: i64) -> SaResult<()> {
        let entry = Arc::new(ObjectEntry {
            value: value.clone(),
            expire_at_ms: timeout_to_deadline_ms(timeout),
        });
        self.objs.insert(key.to_string(), entry).await;
        Ok(())
    }

    async fn update_object(&self, key: &str, value: &serde_json::Value) -> SaResult<()> {
        if let Some(existing) = self.objs.get(key).await {
            self.objs
                .insert(
                    key.to_string(),
                    Arc::new(ObjectEntry {
                        value: value.clone(),
                        expire_at_ms: existing.expire_at_ms,
                    }),
                )
                .await;
        } else {
            self.objs
                .insert(
                    key.to_string(),
                    Arc::new(ObjectEntry {
                        value: value.clone(),
                        expire_at_ms: None,
                    }),
                )
                .await;
        }
        Ok(())
    }

    async fn get_session(&self, session_id: &str) -> SaResult<Option<SaSession>> {
        Ok(touch_session(&self.sessions, session_id)
            .await
            .map(|e| e.session.clone()))
    }

    async fn set_session(&self, session: &SaSession, timeout: i64) -> SaResult<()> {
        let entry = Arc::new(SessionEntry {
            session: session.clone(),
            expire_at_ms: timeout_to_deadline_ms(timeout),
        });
        self.sessions.insert(session.id().to_string(), entry).await;
        Ok(())
    }

    async fn update_session(&self, session: &SaSession) -> SaResult<()> {
        let key = session.id().to_string();
        if let Some(existing) = self.sessions.get(&key).await {
            self.sessions
                .insert(
                    key,
                    Arc::new(SessionEntry {
                        session: session.clone(),
                        expire_at_ms: existing.expire_at_ms,
                    }),
                )
                .await;
        } else {
            self.sessions
                .insert(
                    key,
                    Arc::new(SessionEntry {
                        session: session.clone(),
                        expire_at_ms: None,
                    }),
                )
                .await;
        }
        Ok(())
    }

    async fn search_data(
        &self,
        prefix: &str,
        keyword: &str,
        start: i64,
        size: i64,
        sort_type: bool,
    ) -> SaResult<Vec<String>> {
        let mut keys: Vec<String> = Vec::new();
        for (k, _v) in self.kv.iter() {
            let key = k.as_ref();
            if key.starts_with(prefix)
                && (keyword.is_empty() || key.contains(keyword))
            {
                keys.push(key.to_string());
            }
        }
        for (k, _v) in self.objs.iter() {
            let key = k.as_ref();
            if key.starts_with(prefix)
                && (keyword.is_empty() || key.contains(keyword))
            {
                keys.push(key.to_string());
            }
        }
        for (k, _v) in self.sessions.iter() {
            let key = k.as_ref();
            if key.starts_with(prefix)
                && (keyword.is_empty() || key.contains(keyword))
            {
                keys.push(key.to_string());
            }
        }
        if sort_type {
            keys.sort();
        } else {
            keys.sort_by(|a, b| b.cmp(a));
        }
        let start_idx = start.max(0) as usize;
        let end_idx = if size < 0 {
            keys.len()
        } else {
            (start_idx + size as usize).min(keys.len())
        };
        Ok(keys
            .into_iter()
            .skip(start_idx)
            .take(end_idx - start_idx)
            .collect())
    }
}

/// 同步 API 通过 tokio runtime bridge 调用 moka 异步 API
fn run_on_runtime<F, Fut>(f: F) -> SaResult<()>
where
    F: FnOnce() -> Fut + Send + 'static,
    Fut: std::future::Future<Output = ()> + Send + 'static,
{
    let handle = tokio::runtime::Handle::try_current()
        .map_err(|_| SaTokenException::other("sa-token-dao-moka requires an active tokio runtime"))?;
    tokio::task::block_in_place(|| {
        handle.block_on(f());
    });
    Ok(())
}

#[allow(dead_code)]
fn _ensure_duration() -> Duration {
    Duration::from_secs(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use sa_token_core::config::sa_token_config::SaTokenConfig;
    use sa_token_core::sa_manager::SaManager;
    use std::sync::Arc;

    fn setup() {
        SaManager::reset();
        SaManager::set_config(Arc::new(SaTokenConfig::default()));
        SaManager::set_sa_token_dao(Arc::new(SaTokenDaoMoka::default()));
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn set_get_string_round_trip() {
        setup();
        let dao = SaTokenDaoMoka::default();
        SaTokenDao::set(&dao, "k", "v", -1).expect("set");
        assert_eq!(
            SaTokenDao::get(&dao, "k").expect("get"),
            Some("v".to_string())
        );
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn ttl_expiry_makes_key_invisible() {
        setup();
        let dao = SaTokenDaoMoka::default();
        SaTokenDao::set(&dao, "k", "v", 1).expect("set 1s");
        tokio::time::sleep(std::time::Duration::from_millis(1500)).await;
        assert_eq!(
            SaTokenDao::get(&dao, "k").expect("get after sleep"),
            None
        );
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn object_round_trip() {
        setup();
        let dao = SaTokenDaoMoka::default();
        let value = serde_json::json!({"a": 1, "b": "x"});
        SaTokenDao::set_object(&dao, "obj", &value, -1).expect("set obj");
        assert_eq!(
            SaTokenDao::get_object(&dao, "obj").expect("get obj"),
            Some(value)
        );
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn update_preserves_existing_ttl() {
        setup();
        let dao = SaTokenDaoMoka::default();
        SaTokenDao::set(&dao, "k", "v1", 60).expect("set");
        SaTokenDao::update(&dao, "k", "v2").expect("update");
        assert_eq!(
            SaTokenDao::get(&dao, "k").expect("get after update"),
            Some("v2".to_string())
        );
        let ttl = SaTokenDao::get_timeout(&dao, "k").expect("ttl");
        assert!(ttl > 0 && ttl <= 60, "ttl should be preserved, got {ttl}");
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn capacity_limit_eviction_does_not_crash() {
        setup();
        let dao = SaTokenDaoMoka::builder().max_capacity(2).build();
        SaTokenDao::set(&dao, "a", "1", -1).expect("a");
        SaTokenDao::set(&dao, "b", "2", -1).expect("b");
        SaTokenDao::set(&dao, "c", "3", -1).expect("c");
        dao.maintenance();
        let count = dao.entry_count();
        assert!(count <= 2, "expected at most 2 entries after eviction, got {count}");
    }
}
