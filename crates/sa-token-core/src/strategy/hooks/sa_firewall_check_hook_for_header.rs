//! `SaFirewallCheckHookForHeader` —— 1:1 对应 Java 同名类
//!
//! 防火墙策略校验钩子：请求头检测。

use std::sync::{OnceLock, RwLock};

use crate::context::model::{sa_request::SaRequest, sa_response::SaResponse};
use crate::exception::{FirewallCheckException, SaTokenException};

use super::sa_firewall_check_hook::SaFirewallCheckHook;

/// 请求头校验 hook（对应 Java `SaFirewallCheckHookForHeader`）。
#[derive(Default)]
pub struct SaFirewallCheckHookForHeader {
    /// 不允许出现的请求头名称（对应 Java `notAllowHeaderNames`）。
    pub not_allow_header_names: RwLock<Vec<String>>,
}

impl SaFirewallCheckHookForHeader {
    /// 默认单例（对应 Java `instance`）。
    pub fn instance() -> &'static Self {
        static INST: OnceLock<SaFirewallCheckHookForHeader> = OnceLock::new();
        INST.get_or_init(|| Self {
            not_allow_header_names: RwLock::new(Vec::new()),
        })
    }

    /// 重载配置（对应 Java `resetConfig`）。
    pub fn reset_config(&self, header_names: &[&str]) {
        let mut guard = self.not_allow_header_names.write().unwrap();
        guard.clear();
        guard.extend(header_names.iter().map(|s| (*s).to_string()));
    }
}

impl SaFirewallCheckHook for SaFirewallCheckHookForHeader {
    fn execute(&self, req: &dyn SaRequest, _res: &dyn SaResponse) -> Result<(), SaTokenException> {
        let guard = self.not_allow_header_names.read().unwrap();
        for header_name in guard.iter() {
            if req.get_header(header_name).is_some() {
                return Err(
                    FirewallCheckException::new(format!("非法请求头：{header_name}")).into(),
                );
            }
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
    fn rejects_forbidden_header() {
        let hook = SaFirewallCheckHookForHeader::instance();
        hook.reset_config(&["X-Evil"]);
        let req = SaRequestForMock::new()
            .with_url("/api")
            .with_header("X-Evil", "1");
        assert!(hook.execute(&req, &SaResponseForMock::new()).is_err());
    }
}
