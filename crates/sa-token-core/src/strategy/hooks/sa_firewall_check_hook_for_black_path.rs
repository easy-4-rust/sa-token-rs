//! `SaFirewallCheckHookForBlackPath` —— 1:1 对应 Java 同名类
//!
//! 防火墙策略校验钩子：请求 path 黑名单校验。

use std::sync::{OnceLock, RwLock};

use crate::context::model::{sa_request::SaRequest, sa_response::SaResponse};
use crate::exception::{RequestPathInvalidException, SaTokenException};

use super::sa_firewall_check_hook::SaFirewallCheckHook;

/// 黑名单 path hook（对应 Java `SaFirewallCheckHookForBlackPath`）。
pub struct SaFirewallCheckHookForBlackPath {
    /// 黑名单 path 列表（对应 Java `blackPaths`）。
    pub black_paths: RwLock<Vec<String>>,
}

impl SaFirewallCheckHookForBlackPath {
    /// 默认单例（对应 Java `instance`）。
    pub fn instance() -> &'static Self {
        static INST: OnceLock<SaFirewallCheckHookForBlackPath> = OnceLock::new();
        INST.get_or_init(|| Self {
            black_paths: RwLock::new(Vec::new()),
        })
    }

    /// 重载配置（对应 Java `resetConfig`）。
    pub fn reset_config(&self, paths: &[&str]) {
        let mut guard = self.black_paths.write().unwrap();
        guard.clear();
        guard.extend(paths.iter().map(|s| (*s).to_string()));
    }
}

impl SaFirewallCheckHook for SaFirewallCheckHookForBlackPath {
    fn execute(&self, req: &dyn SaRequest, _res: &dyn SaResponse) -> Result<(), SaTokenException> {
        let request_path = req.get_request_path();
        let guard = self.black_paths.read().unwrap();
        if guard.iter().any(|item| request_path == *item) {
            return Err(RequestPathInvalidException::new(format!(
                "非法请求：{request_path}"
            ))
            .into());
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context::mock::sa_request_for_mock::SaRequestForMock;
    use crate::context::mock::sa_response_for_mock::SaResponseForMock;

    #[test]
    fn rejects_blacklisted_path() {
        let hook = SaFirewallCheckHookForBlackPath::instance();
        hook.reset_config(&["/admin/secret"]);
        let req = SaRequestForMock::new().with_url("/admin/secret");
        assert!(hook.execute(&req, &SaResponseForMock::new()).is_err());
    }
}
