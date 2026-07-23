//! `SaFirewallCheckHookForParameter` —— 1:1 对应 Java 同名类
//!
//! 防火墙策略校验钩子：请求参数检测。

use std::sync::{OnceLock, RwLock};

use crate::context::model::{sa_request::SaRequest, sa_response::SaResponse};
use crate::exception::{FirewallCheckException, SaTokenException};

use super::sa_firewall_check_hook::SaFirewallCheckHook;

/// 请求参数校验 hook（对应 Java `SaFirewallCheckHookForParameter`）。
#[derive(Default)]
pub struct SaFirewallCheckHookForParameter {
    /// 不允许出现的参数名（对应 Java `notAllowParameterNames`）。
    pub not_allow_parameter_names: RwLock<Vec<String>>,
}

impl SaFirewallCheckHookForParameter {
    /// 默认单例（对应 Java `instance`）。
    pub fn instance() -> &'static Self {
        static INST: OnceLock<SaFirewallCheckHookForParameter> = OnceLock::new();
        INST.get_or_init(|| Self {
            not_allow_parameter_names: RwLock::new(Vec::new()),
        })
    }

    /// 重载配置（对应 Java `resetConfig`）。
    pub fn reset_config(&self, parameter_names: &[&str]) {
        let mut guard = self.not_allow_parameter_names.write().unwrap();
        guard.clear();
        guard.extend(parameter_names.iter().map(|s| (*s).to_string()));
    }
}

impl SaFirewallCheckHook for SaFirewallCheckHookForParameter {
    fn execute(&self, req: &dyn SaRequest, _res: &dyn SaResponse) -> Result<(), SaTokenException> {
        let guard = self.not_allow_parameter_names.read().unwrap();
        for parameter_name in guard.iter() {
            if req.get_param(parameter_name).is_some() {
                return Err(
                    FirewallCheckException::new(format!("非法请求参数：{parameter_name}")).into(),
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
    fn rejects_forbidden_parameter() {
        let hook = SaFirewallCheckHookForParameter::instance();
        hook.reset_config(&["debug"]);
        let req = SaRequestForMock::new()
            .with_url("/api")
            .with_query("debug", "1");
        assert!(hook.execute(&req, &SaResponseForMock::new()).is_err());
    }
}
