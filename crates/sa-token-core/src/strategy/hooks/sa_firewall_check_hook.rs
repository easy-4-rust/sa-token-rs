//! `SaFirewallCheckHook` —— 1:1 对应 Java `cn.dev33.satoken.strategy.hooks.SaFirewallCheckHook`
//!
//! 防火墙策略校验钩子函数接口。

use crate::context::model::{sa_request::SaRequest, sa_response::SaResponse};
use crate::exception::SaTokenException;

/// 防火墙校验钩子（对应 Java `@FunctionalInterface SaFirewallCheckHook`）。
pub trait SaFirewallCheckHook: Send + Sync + 'static {
    /// 执行校验逻辑（对应 Java `execute(SaRequest req, SaResponse res, Object extArg)`）。
    fn execute(&self, req: &dyn SaRequest, res: &dyn SaResponse) -> Result<(), SaTokenException>;
}

/// 包装静态单例 hook，便于注册到策略链。
pub struct StaticHookWrapper<H: SaFirewallCheckHook + ?Sized + 'static> {
    inner: &'static H,
}

impl<H: SaFirewallCheckHook + ?Sized + 'static> StaticHookWrapper<H> {
    /// 创建静态 hook 包装器。
    pub fn new(inner: &'static H) -> Self {
        Self { inner }
    }
}

impl<H: SaFirewallCheckHook + ?Sized + 'static> SaFirewallCheckHook for StaticHookWrapper<H> {
    fn execute(&self, req: &dyn SaRequest, res: &dyn SaResponse) -> Result<(), SaTokenException> {
        self.inner.execute(req, res)
    }
}
