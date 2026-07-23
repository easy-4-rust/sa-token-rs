//! `SaFirewallCheckHookForHost` —— 1:1 对应 Java 同名类
//!
//! 防火墙策略校验钩子：Host 检测。

use std::sync::{OnceLock, RwLock};

use crate::context::model::{sa_request::SaRequest, sa_response::SaResponse};
use crate::exception::{FirewallCheckException, SaTokenException};
use crate::strategy::sa_strategy::SaStrategy;

use super::sa_firewall_check_hook::SaFirewallCheckHook;

/// Host 校验 hook（对应 Java `SaFirewallCheckHookForHost`）。
#[derive(Default)]
pub struct SaFirewallCheckHookForHost {
    /// 是否校验 host（对应 Java `isCheckHost`）。
    pub is_check_host: RwLock<bool>,
    /// 允许的 host 列表（对应 Java `allowHosts`）。
    pub allow_hosts: RwLock<Vec<String>>,
}

impl SaFirewallCheckHookForHost {
    /// 默认单例（对应 Java `instance`）。
    pub fn instance() -> &'static Self {
        static INST: OnceLock<SaFirewallCheckHookForHost> = OnceLock::new();
        INST.get_or_init(|| Self {
            is_check_host: RwLock::new(false),
            allow_hosts: RwLock::new(Vec::new()),
        })
    }

    /// 重载配置（对应 Java `resetConfig`）。
    pub fn reset_config(&self, is_check_host: bool, allow_hosts: &[&str]) {
        *self.is_check_host.write().unwrap() = is_check_host;
        let mut guard = self.allow_hosts.write().unwrap();
        guard.clear();
        guard.extend(allow_hosts.iter().map(|s| (*s).to_string()));
    }
}

impl SaFirewallCheckHook for SaFirewallCheckHookForHost {
    fn execute(&self, req: &dyn SaRequest, _res: &dyn SaResponse) -> Result<(), SaTokenException> {
        if !*self.is_check_host.read().unwrap() {
            return Ok(());
        }
        let host = req.get_host();
        let allow = self.allow_hosts.read().unwrap().clone();
        let has_element = SaStrategy::instance().has_element.as_ref();
        if !has_element(&allow, &host) {
            return Err(FirewallCheckException::new(format!("非法请求 host：{host}")).into());
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
    fn rejects_unknown_host_when_enabled() {
        let hook = SaFirewallCheckHookForHost::instance();
        hook.reset_config(true, &["localhost"]);
        let req = SaRequestForMock::new()
            .with_url("/api")
            .with_host("evil.example");
        assert!(hook.execute(&req, &SaResponseForMock::new()).is_err());
    }
}
