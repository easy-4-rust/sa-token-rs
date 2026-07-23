//! `SaFirewallStrategy` —— 1:1 对应 Java `cn.dev33.satoken.strategy.SaFirewallStrategy`
//!
//! Sa-Token 防火墙策略：注册并执行一组校验 hook。

use std::sync::{OnceLock, RwLock};

use crate::context::model::{sa_request::SaRequest, sa_response::SaResponse};
use crate::exception::SaTokenException;
use crate::fun::strategy::sa_firewall_check_fail_handle_function::SaFirewallCheckFailHandleFunction;
use crate::fun::strategy::sa_firewall_check_function::SaFirewallCheckFunction;

use super::hooks::{
    sa_firewall_check_hook::{SaFirewallCheckHook, StaticHookWrapper},
    sa_firewall_check_hook_for_black_path::SaFirewallCheckHookForBlackPath,
    sa_firewall_check_hook_for_directory_traversal::SaFirewallCheckHookForDirectoryTraversal,
    sa_firewall_check_hook_for_header::SaFirewallCheckHookForHeader,
    sa_firewall_check_hook_for_host::SaFirewallCheckHookForHost,
    sa_firewall_check_hook_for_http_method::SaFirewallCheckHookForHttpMethod,
    sa_firewall_check_hook_for_parameter::SaFirewallCheckHookForParameter,
    sa_firewall_check_hook_for_path_banned_character::SaFirewallCheckHookForPathBannedCharacter,
    sa_firewall_check_hook_for_path_danger_character::SaFirewallCheckHookForPathDangerCharacter,
    sa_firewall_check_hook_for_white_path::SaFirewallCheckHookForWhitePath,
};

/// 防火墙策略（对应 Java `SaFirewallStrategy.instance`）。
pub struct SaFirewallStrategy {
    /// 防火墙校验 hook 集合（对应 Java `checkHooks`）。
    pub check_hooks: RwLock<Vec<Box<dyn SaFirewallCheckHook>>>,
    /// 防火墙校验函数（对应 Java `check`）。
    pub check: SaFirewallCheckFunction,
    /// 校验失败处理函数（对应 Java `checkFailHandle`）。
    pub check_fail_handle: RwLock<Option<SaFirewallCheckFailHandleFunction>>,
}

impl SaFirewallStrategy {
    /// 创建带默认 hook 的策略实例（对应 Java 私有构造器）。
    fn new() -> Self {
        let mut hooks: Vec<Box<dyn SaFirewallCheckHook>> = Vec::new();
        hooks.push(Box::new(StaticHookWrapper::new(
            SaFirewallCheckHookForWhitePath::instance(),
        )));
        hooks.push(Box::new(StaticHookWrapper::new(
            SaFirewallCheckHookForBlackPath::instance(),
        )));
        hooks.push(Box::new(StaticHookWrapper::new(
            SaFirewallCheckHookForPathDangerCharacter::instance(),
        )));
        hooks.push(Box::new(StaticHookWrapper::new(
            SaFirewallCheckHookForPathBannedCharacter::instance(),
        )));
        hooks.push(Box::new(StaticHookWrapper::new(
            SaFirewallCheckHookForDirectoryTraversal::instance(),
        )));
        hooks.push(Box::new(StaticHookWrapper::new(
            SaFirewallCheckHookForHost::instance(),
        )));
        hooks.push(Box::new(StaticHookWrapper::new(
            SaFirewallCheckHookForHttpMethod::instance(),
        )));
        hooks.push(Box::new(StaticHookWrapper::new(
            SaFirewallCheckHookForHeader::instance(),
        )));
        hooks.push(Box::new(StaticHookWrapper::new(
            SaFirewallCheckHookForParameter::instance(),
        )));

        Self {
            check_hooks: RwLock::new(hooks),
            check: Box::new(Self::default_check),
            check_fail_handle: RwLock::new(None),
        }
    }

    /// 默认防火墙校验入口（对应 Java `check` 字段的 lambda）。
    fn default_check(
        req: &dyn SaRequest,
        res: &dyn SaResponse,
    ) -> Result<(), SaTokenException> {
        Self::instance().execute_check(req, res)
    }

    /// 获取全局单例（对应 Java `SaFirewallStrategy.instance`）。
    pub fn instance() -> &'static SaFirewallStrategy {
        static INST: OnceLock<SaFirewallStrategy> = OnceLock::new();
        INST.get_or_init(SaFirewallStrategy::new)
    }

    /// 注册 hook 到末尾（对应 Java `registerHook`）。
    pub fn register_hook(&self, hook: Box<dyn SaFirewallCheckHook>) {
        self.check_hooks.write().unwrap().push(hook);
    }

    /// 注册 hook 到首位（对应 Java `registerHookToFirst`）。
    pub fn register_hook_to_first(&self, hook: Box<dyn SaFirewallCheckHook>) {
        self.check_hooks.write().unwrap().insert(0, hook);
    }

    /// 注册 hook 到第二位（对应 Java `registerHookToSecond`）。
    pub fn register_hook_to_second(&self, hook: Box<dyn SaFirewallCheckHook>) {
        self.check_hooks.write().unwrap().insert(1, hook);
    }

    /// 执行当前已注册的 hook 链（供测试与内部调用）。
    pub fn execute_check(
        &self,
        req: &dyn SaRequest,
        res: &dyn SaResponse,
    ) -> Result<(), SaTokenException> {
        let hooks = self.check_hooks.read().unwrap();
        Self::run_hooks(req, res, &hooks)
    }

    /// 依次执行 hook；白名单命中时抛出 `StopMatch` 视为放行。
    fn run_hooks(
        req: &dyn SaRequest,
        res: &dyn SaResponse,
        hooks: &[Box<dyn SaFirewallCheckHook>],
    ) -> Result<(), SaTokenException> {
        for hook in hooks {
            match hook.execute(req, res) {
                Ok(()) => {}
                Err(SaTokenException::StopMatch) => return Ok(()),
                Err(err) => return Err(err),
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
    use crate::exception::SaTokenException;

    #[test]
    fn directory_traversal_hook_rejects_dot_dot() {
        let req = SaRequestForMock::new().with_url("/user/../info");
        let res = SaResponseForMock::new();
        let err = SaFirewallCheckHookForDirectoryTraversal::instance()
            .execute(&req, &res)
            .unwrap_err();
        assert!(matches!(
            err,
            SaTokenException::RequestPathInvalid { .. }
        ));
    }

    #[test]
    fn white_path_hook_stops_on_match() {
        SaFirewallCheckHookForWhitePath::instance().reset_config(&["/public"]);
        let req = SaRequestForMock::new().with_url("/public");
        let res = SaResponseForMock::new();
        let err = SaFirewallCheckHookForWhitePath::instance()
            .execute(&req, &res)
            .unwrap_err();
        assert_eq!(err, SaTokenException::StopMatch);
    }
}
