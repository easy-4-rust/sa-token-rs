//! `SaFirewallCheckHookForHttpMethod` —— 1:1 对应 Java 同名类
//!
//! 防火墙策略校验钩子：请求 Method 检测。

use std::sync::{OnceLock, RwLock};

use crate::context::model::{sa_request::SaRequest, sa_response::SaResponse};
use crate::exception::{FirewallCheckException, SaTokenException};
use crate::router::sa_http_method::SaHttpMethod;

use super::sa_firewall_check_hook::SaFirewallCheckHook;

/// HTTP Method 校验 hook（对应 Java `SaFirewallCheckHookForHttpMethod`）。
#[derive(Default)]
pub struct SaFirewallCheckHookForHttpMethod {
    /// 是否校验 Method（对应 Java `isCheckMethod`）。
    pub is_check_method: RwLock<bool>,
    /// 允许的 Method 列表（对应 Java `allowMethods`）。
    pub allow_methods: RwLock<Vec<String>>,
}

impl SaFirewallCheckHookForHttpMethod {
    /// 默认单例（对应 Java `instance`）。
    pub fn instance() -> &'static Self {
        static INST: OnceLock<SaFirewallCheckHookForHttpMethod> = OnceLock::new();
        INST.get_or_init(|| {
            let methods = [
                SaHttpMethod::Get,
                SaHttpMethod::Post,
                SaHttpMethod::Put,
                SaHttpMethod::Delete,
                SaHttpMethod::Head,
                SaHttpMethod::Options,
                SaHttpMethod::Patch,
                SaHttpMethod::Trace,
                SaHttpMethod::Connect,
            ]
            .into_iter()
            .map(|m| m.to_string())
            .collect();
            Self {
                is_check_method: RwLock::new(true),
                allow_methods: RwLock::new(methods),
            }
        })
    }

    /// 重载配置（对应 Java `resetConfig`）。
    pub fn reset_config(&self, is_check_method: bool, methods: &[&str]) {
        *self.is_check_method.write().unwrap() = is_check_method;
        let mut guard = self.allow_methods.write().unwrap();
        guard.clear();
        guard.extend(methods.iter().map(|s| (*s).to_string()));
    }
}

impl SaFirewallCheckHook for SaFirewallCheckHookForHttpMethod {
    fn execute(&self, req: &dyn SaRequest, _res: &dyn SaResponse) -> Result<(), SaTokenException> {
        if !*self.is_check_method.read().unwrap() {
            return Ok(());
        }
        let method = req.get_method();
        let allow = self.allow_methods.read().unwrap();
        if !allow.iter().any(|item| item.eq_ignore_ascii_case(&method)) {
            return Err(FirewallCheckException::new(format!("非法请求 Method：{method}")).into());
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
    fn rejects_unknown_method() {
        let hook = SaFirewallCheckHookForHttpMethod::instance();
        hook.reset_config(true, &["GET", "POST"]);
        let req = SaRequestForMock::new().with_url("/api").with_method("FOO");
        assert!(hook.execute(&req, &SaResponseForMock::new()).is_err());
    }
}
