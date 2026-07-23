//! `SaFirewallCheckHookForWhitePath` —— 1:1 对应 Java 同名类
//!
//! 防火墙策略校验钩子：请求 path 白名单放行。

use std::sync::{OnceLock, RwLock};

use crate::context::model::{sa_request::SaRequest, sa_response::SaResponse};
use crate::exception::{SaTokenException, StopMatchException};

use super::sa_firewall_check_hook::SaFirewallCheckHook;

/// 白名单 path hook（对应 Java `SaFirewallCheckHookForWhitePath`）。
pub struct SaFirewallCheckHookForWhitePath {
    /// 白名单 path 列表（对应 Java `whitePaths`）。
    pub white_paths: RwLock<Vec<String>>,
}

impl SaFirewallCheckHookForWhitePath {
    /// 默认单例（对应 Java `instance`）。
    pub fn instance() -> &'static Self {
        static INST: OnceLock<SaFirewallCheckHookForWhitePath> = OnceLock::new();
        INST.get_or_init(|| Self {
            white_paths: RwLock::new(Vec::new()),
        })
    }

    /// 重载配置（对应 Java `resetConfig`）。
    pub fn reset_config(&self, paths: &[&str]) {
        let mut guard = self.white_paths.write().unwrap();
        guard.clear();
        guard.extend(paths.iter().map(|s| (*s).to_string()));
    }
}

impl SaFirewallCheckHook for SaFirewallCheckHookForWhitePath {
    /// 命中白名单则抛出 `StopMatchException` 跳过后续 hook。
    fn execute(&self, req: &dyn SaRequest, _res: &dyn SaResponse) -> Result<(), SaTokenException> {
        let request_path = req.get_request_path();
        let guard = self.white_paths.read().unwrap();
        if guard.iter().any(|item| request_path == *item) {
            return Err(StopMatchException::new().into());
        }
        Ok(())
    }
}
