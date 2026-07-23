//! Quick login configuration (Java `SaQuickConfig`).

use std::sync::Arc;

use sa_token_core::stp::stp_util::StpUtil;
use sa_token_core::util::sa_fox_util;
use sa_token_core::util::sa_result::SaResultData;
use serde_json::Value;

use crate::quick::function::do_login_handle_function::DoLoginHandleFunction;

/// Quick login configuration model.
#[derive(Clone)]
pub struct SaQuickConfig {
    /// Whether auth interception is enabled.
    pub auth: bool,
    /// Username.
    pub name: String,
    /// Password.
    pub pwd: String,
    /// Auto-generate random username/password.
    pub auto: bool,
    /// Login page title.
    pub title: String,
    /// Show copyright footer.
    pub copr: bool,
    /// Comma-separated include patterns.
    pub include: String,
    /// Comma-separated exclude patterns.
    pub exclude: String,
    do_login_handle: Option<Arc<dyn DoLoginHandleFunction>>,
}

impl Default for SaQuickConfig {
    fn default() -> Self {
        Self {
            auth: true,
            name: "sa".into(),
            pwd: "123456".into(),
            auto: false,
            title: "Sa-Token 登录".into(),
            copr: true,
            include: "/**".into(),
            exclude: String::new(),
            do_login_handle: None,
        }
    }
}

impl SaQuickConfig {
    /// Replaces the login handler.
    pub fn with_do_login_handle(
        mut self,
        handler: Arc<dyn DoLoginHandleFunction>,
    ) -> Self {
        self.do_login_handle = Some(handler);
        self
    }

    /// Applies configured login handler or built-in default.
    pub fn do_login(&self, name: &str, pwd: &str) -> SaResultData<Value> {
        if let Some(handler) = &self.do_login_handle {
            return handler.apply(name, pwd);
        }
        self.default_do_login(name, pwd)
    }

    /// Built-in credential check using configured `name` / `pwd`.
    pub fn default_do_login(&self, name: &str, pwd: &str) -> SaResultData<Value> {
        if sa_fox_util::is_empty(name) || sa_fox_util::is_empty(pwd) {
            return SaResultData::error(500, "请输入账号和密码");
        }
        if name == self.name && pwd == self.pwd {
            match StpUtil::login(name).and_then(|_| StpUtil::get_token_info()) {
                Ok(info) => match serde_json::to_value(info) {
                    Ok(data) => SaResultData::ok(data),
                    Err(error) => SaResultData::error(500, error.to_string()),
                },
                Err(error) => SaResultData::error(500, error.to_string()),
            }
        } else {
            SaResultData::error(500, "账号或密码输入错误")
        }
    }

    /// Applies `auto` flag side effects (random account generation).
    pub fn apply_auto_credentials(&mut self) {
        if self.auto {
            self.name = sa_fox_util::random_string(8);
            self.pwd = sa_fox_util::random_string(8);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sa_token_core::config::sa_token_config::SaTokenConfig;
    use sa_token_core::context::mock::sa_token_context_mock_util::SaTokenContextMockUtil;
    use sa_token_core::sa_manager::SaManager;
    use sa_token_core::stp::stp_logic::StpLogic;
    use sa_token_dao_memory::SaTokenDaoMemory;
    use std::sync::{Arc, Mutex};

    /// 串行化依赖全局 `SaManager` 的单测，避免与集成测试竞态。
    static TEST_LOCK: Mutex<()> = Mutex::new(());

    fn setup() -> std::sync::MutexGuard<'static, ()> {
        let guard = TEST_LOCK
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        SaManager::reset();
        SaManager::set_config(Arc::new(SaTokenConfig::default()));
        SaManager::set_sa_token_dao(Arc::new(SaTokenDaoMemory::new()));
        SaTokenContextMockUtil::set_mock_context();
        SaManager::put_stp_logic(Arc::new(StpLogic::new("login")));
        guard
    }

    #[test]
    fn default_login_success_and_failure() {
        let _guard = setup();
        let cfg = SaQuickConfig::default();
        let ok = cfg.do_login("sa", "123456");
        assert!(
            ok.is_ok(),
            "expected login ok, got code={} message={}",
            ok.code,
            ok.message
        );
        let bad = cfg.do_login("sa", "bad");
        assert!(!bad.is_ok());
    }

    #[test]
    fn auto_generates_credentials() {
        let mut cfg = SaQuickConfig {
            auto: true,
            ..Default::default()
        };
        cfg.apply_auto_credentials();
        assert_eq!(cfg.name.len(), 8);
        assert_eq!(cfg.pwd.len(), 8);
    }
}
