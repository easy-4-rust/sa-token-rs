//! `sa-token-remember-me` —— Remember-me 插件。
//!
//! 对应 Java 端 `sa-token-demo-remember-me`（一个完整 demo 子项目，
//! Java 端没把它升级为独立 plugin）。本 crate 把 remember-me 业务逻辑
//! 抽成可复用 plugin：
//!
//! 1. `RememberMeManager::issue()` —— 登录后生成长期"记住我" token
//!    （独立的 token 池，与短期 token 不共享生命周期）
//! 2. `RememberMeManager::resolve()` —— 解析 remember-me token
//! 3. `RememberMeConfig` —— 配置项（cookie 名、过期时间、设备限制）
//!
//! # 数据布局
//!
//! | key                                    | value            |
//! |----------------------------------------|------------------|
//! | `{prefix}:token:{token_value}`         | `{login_id}`     |
//! | `{prefix}:login:{login_id}:tokens`     | `[token,...]` JSON 列表 |
//!
//! 通过 token 自身作为 key，避免"覆盖式存储"导致的多 token 丢失；
//! login_id 反向索引为 JSON 数组以正确跟踪 token 数量上限。
//!
//! # 用法
//! ```rust,ignore
//! use sa_token_remember_me::{RememberMeManager, RememberMeConfig};
//!
//! let config = RememberMeConfig::default()
//!     .with_cookie_name("remember-me")
//!     .with_timeout(60 * 60 * 24 * 7); // 7 天
//! let manager = RememberMeManager::new(config);
//!
//! // 登录成功后：
//! let remember_token = manager.issue("user-10001")?;
//!
//! // 客户端下次请求携带 remember-me cookie：
//! let login_id = manager.resolve(&remember_token)?;
//! ```

use std::sync::Arc;

use sa_token_core::exception::{SaResult, SaTokenException};
use sa_token_core::sa_manager::SaManager;

/// Remember-me 配置
#[derive(Clone, Debug)]
pub struct RememberMeConfig {
    /// 存储 key 前缀（用于在 DAO 中区分 remember-me token）
    pub key_prefix: String,
    /// 过期时间（秒），默认 7 天
    pub timeout: i64,
    /// 同一账号最大 remember-me token 数（-1 = 不限）
    pub max_per_account: i32,
    /// 是否限制单设备（true = 同一账号只有一个 remember-me 有效）
    pub single_device: bool,
}

impl Default for RememberMeConfig {
    fn default() -> Self {
        Self {
            key_prefix: "remember-me".to_string(),
            timeout: 60 * 60 * 24 * 7,
            max_per_account: 5,
            single_device: false,
        }
    }
}

impl RememberMeConfig {
    pub fn with_cookie_name(mut self, name: impl Into<String>) -> Self {
        self.key_prefix = name.into();
        self
    }

    pub fn with_timeout(mut self, timeout: i64) -> Self {
        self.timeout = timeout;
        self
    }

    pub fn with_max_per_account(mut self, max: i32) -> Self {
        self.max_per_account = max;
        self
    }

    pub fn with_single_device(mut self, single: bool) -> Self {
        self.single_device = single;
        self
    }
}

/// Remember-me manager
#[derive(Clone)]
pub struct RememberMeManager {
    config: RememberMeConfig,
}

impl RememberMeManager {
    pub fn new(config: RememberMeConfig) -> Self {
        Self { config }
    }

    /// 全局默认 manager（从 SaManager 提取 token name 作为 key 前缀）
    pub fn from_global_default() -> Self {
        let cfg = SaManager::config();
        let prefix = format!("{}:remember-me", cfg.token_name());
        Self::new(RememberMeConfig {
            key_prefix: prefix,
            ..Default::default()
        })
    }

    /// 单 token 存储 key：`{prefix}:token:{token_value}`
    fn splicing_token_key(&self, token: &str) -> String {
        format!("{}:token:{}", self.config.key_prefix, token)
    }

    /// login_id 反向索引 key：`{prefix}:login:{login_id}:tokens`
    fn splicing_login_index_key(&self, login_id: &str) -> String {
        format!("{}:login:{}:tokens", self.config.key_prefix, login_id)
    }

    /// 签发一个 remember-me token
    ///
    /// 存储到 DAO：key = `{prefix}:token:{token_value}`，value = login_id
    /// 同时维护 `{prefix}:login:{login_id}:tokens` 的反向索引。
    pub fn issue(&self, login_id: &str) -> SaResult<String> {
        if login_id.is_empty() {
            return Err(SaTokenException::NotLogin {
                message: "login_id 不能为空".to_string(),
                login_type: "remember-me".to_string(),
                scene: "NOT_TOKEN".to_string(),
                code: 0,
            });
        }
        let dao = SaManager::sa_token_dao();

        // single_device 模式：先撤销已有 token
        if self.config.single_device {
            self.revoke_all_for_login(login_id)?;
        }

        // 检查 max_per_account
        if self.config.max_per_account > 0 {
            let existing_count = self.count_tokens_for_login(login_id);
            if existing_count >= self.config.max_per_account {
                return Err(SaTokenException::Other {
                    message: format!(
                        "remember-me 账号 {login_id} 已达上限 {}",
                        self.config.max_per_account
                    ),
                });
            }
        }

        // 生成唯一 token（不依赖 login_id 重复值）
        let token = uuid_v4_like();
        dao.set(&self.splicing_token_key(&token), login_id, self.config.timeout)?;

        // 更新反向索引：把新 token 加入 login_id 的 token 列表
        let mut tokens = self.list_tokens_for_login(login_id);
        tokens.push(token.clone());
        let tokens_json = serde_json::to_string(&tokens).map_err(|e| SaTokenException::Other {
            message: format!("serialize tokens: {e}"),
        })?;
        dao.set(
            &self.splicing_login_index_key(login_id),
            &tokens_json,
            self.config.timeout,
        )?;

        Ok(token)
    }

    /// 解析 remember-me token → login_id
    pub fn resolve(&self, token: &str) -> SaResult<Option<String>> {
        if token.is_empty() {
            return Ok(None);
        }
        let dao = SaManager::sa_token_dao();
        // token 自身作为 key，O(1) 查找
        Ok(dao.get(&self.splicing_token_key(token))?)
    }

    /// 撤销 remember-me token（按 token value）
    pub fn revoke_token(&self, token: &str) -> SaResult<()> {
        if token.is_empty() {
            return Ok(());
        }
        let dao = SaManager::sa_token_dao();
        if let Some(login_id) = dao.get(&self.splicing_token_key(token))? {
            // 删除 token
            dao.delete(&self.splicing_token_key(token))?;
            // 从反向索引移除
            self.remove_token_from_index(&login_id, token)?;
        }
        Ok(())
    }

    /// 撤销某个 login_id 的所有 remember-me token
    pub fn revoke(&self, login_id: &str) -> SaResult<()> {
        self.revoke_all_for_login(login_id)
    }

    /// 内部 helper：撤销一个 login_id 的全部 token
    fn revoke_all_for_login(&self, login_id: &str) -> SaResult<()> {
        let dao = SaManager::sa_token_dao();
        for token in self.list_tokens_for_login(login_id) {
            dao.delete(&self.splicing_token_key(&token))?;
        }
        dao.delete(&self.splicing_login_index_key(login_id))?;
        Ok(())
    }

    /// 内部 helper：从反向索引中移除一个 token
    fn remove_token_from_index(&self, login_id: &str, token: &str) -> SaResult<()> {
        let dao = SaManager::sa_token_dao();
        let idx_key = self.splicing_login_index_key(login_id);
        let mut tokens = self.list_tokens_for_login(login_id);
        tokens.retain(|t| t != token);
        if tokens.is_empty() {
            dao.delete(&idx_key)?;
        } else {
            let tokens_json = serde_json::to_string(&tokens).map_err(|e| {
                SaTokenException::Other {
                    message: format!("serialize tokens: {e}"),
                }
            })?;
            dao.set(&idx_key, &tokens_json, self.config.timeout)?;
        }
        Ok(())
    }

    /// 内部 helper：列出 login_id 的所有 token
    fn list_tokens_for_login(&self, login_id: &str) -> Vec<String> {
        let dao = SaManager::sa_token_dao();
        let idx_key = self.splicing_login_index_key(login_id);
        dao.get(&idx_key)
            .ok()
            .flatten()
            .and_then(|raw| serde_json::from_str::<Vec<String>>(&raw).ok())
            .unwrap_or_default()
    }

    /// 内部 helper：统计 login_id 的 token 数
    fn count_tokens_for_login(&self, login_id: &str) -> i32 {
        self.list_tokens_for_login(login_id).len() as i32
    }
}

/// 简易 UUID v4 生成（避免引入额外依赖）
fn uuid_v4_like() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    format!("{nanos:x}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use sa_token_core::config::sa_token_config::SaTokenConfig;
    use sa_token_core::dao::sa_token_dao_default_impl::SaTokenDaoDefaultImpl;
    use sa_token_core::stp::stp_logic::StpLogic;

    fn setup() {
        SaManager::reset();
        SaManager::set_config(Arc::new(SaTokenConfig::default()));
        SaManager::set_sa_token_dao(Arc::new(SaTokenDaoDefaultImpl::new()));
        SaManager::put_stp_logic(Arc::new(StpLogic::new("login")));
    }

    #[test]
    fn config_default_values() {
        let cfg = RememberMeConfig::default();
        assert_eq!(cfg.key_prefix, "remember-me");
        assert_eq!(cfg.timeout, 60 * 60 * 24 * 7);
        assert_eq!(cfg.max_per_account, 5);
    }

    #[test]
    fn config_builder_methods() {
        let cfg = RememberMeConfig::default()
            .with_cookie_name("rm")
            .with_timeout(60)
            .with_max_per_account(1)
            .with_single_device(true);
        assert_eq!(cfg.key_prefix, "rm");
        assert_eq!(cfg.timeout, 60);
        assert_eq!(cfg.max_per_account, 1);
        assert!(cfg.single_device);
    }

    #[test]
    fn issue_and_resolve_token() {
        setup();
        let manager = RememberMeManager::new(RememberMeConfig::default());
        let token = manager.issue("user-1").expect("issue");
        // token 是 uuid，不带前缀（已重构）
        assert!(!token.is_empty());
        let resolved = manager.resolve(&token).expect("resolve");
        assert_eq!(resolved.as_deref(), Some("user-1"));
    }

    #[test]
    fn resolve_unknown_token_returns_none() {
        setup();
        let manager = RememberMeManager::new(RememberMeConfig::default());
        let resolved = manager.resolve("unknown-token-123").expect("resolve");
        assert!(resolved.is_none());
    }

    #[test]
    fn revoke_removes_token() {
        setup();
        let manager = RememberMeManager::new(RememberMeConfig::default());
        let token = manager.issue("user-2").expect("issue");
        manager.revoke("user-2").expect("revoke");
        let resolved = manager.resolve(&token).expect("resolve");
        assert!(resolved.is_none());
    }

    #[test]
    fn max_per_account_limit() {
        setup();
        let manager = RememberMeManager::new(
            RememberMeConfig::default().with_max_per_account(1),
        );
        manager.issue("user-3").expect("first");
        let second = manager.issue("user-3");
        assert!(second.is_err(), "第二次应被 max_per_account 限制拒绝");
    }

    #[test]
    fn issue_multiple_tokens_below_limit() {
        setup();
        let manager = RememberMeManager::new(
            RememberMeConfig::default().with_max_per_account(3),
        );
        let t1 = manager.issue("user-4").expect("first");
        let t2 = manager.issue("user-4").expect("second");
        let t3 = manager.issue("user-4").expect("third");
        // 每个 token 都应能独立解析回同一个 login_id
        assert_eq!(manager.resolve(&t1).unwrap().as_deref(), Some("user-4"));
        assert_eq!(manager.resolve(&t2).unwrap().as_deref(), Some("user-4"));
        assert_eq!(manager.resolve(&t3).unwrap().as_deref(), Some("user-4"));
        // 撤销其中一个不影响其他
        manager.revoke_token(&t2).expect("revoke t2");
        assert!(manager.resolve(&t2).unwrap().is_none());
        assert_eq!(manager.resolve(&t1).unwrap().as_deref(), Some("user-4"));
        assert_eq!(manager.resolve(&t3).unwrap().as_deref(), Some("user-4"));
    }

    #[test]
    fn single_device_mode_evicts_previous_token() {
        setup();
        let manager = RememberMeManager::new(
            RememberMeConfig::default().with_single_device(true),
        );
        let t1 = manager.issue("user-5").expect("first");
        let t2 = manager.issue("user-5").expect("second");
        // 第二个 token 应使第一个失效
        assert!(manager.resolve(&t1).unwrap().is_none(), "first token 应被 second 顶掉");
        assert_eq!(manager.resolve(&t2).unwrap().as_deref(), Some("user-5"));
    }

    #[test]
    fn issue_with_empty_login_id_errors() {
        setup();
        let manager = RememberMeManager::new(RememberMeConfig::default());
        let result = manager.issue("");
        assert!(result.is_err());
    }

    #[test]
    fn empty_token_resolve_returns_none() {
        setup();
        let manager = RememberMeManager::new(RememberMeConfig::default());
        assert!(manager.resolve("").unwrap().is_none());
    }
}
