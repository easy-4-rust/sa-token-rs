//! `sa-token-web-rocket` —— Rocket 框架适配层。
//!
//! 提供：
//! - `RocketContext`：`SaTokenContext` 的 Rocket 实现（通过 Rocket request guard 包装）
//! - `extract_login_id` / `extract_optional_login_id`：便捷提取器
//!
//! 与 axum/poem 适配器保持 API 形态一致。Rocket 的请求生命周期由
//! `FromRequest` / `request::Outcome` 驱动，因此本 crate 在生产代码中
//! 应配合 Rocket 的 `&Request<...>` guard 使用。
//!
//! 注：Rocket 0.5 是异步运行时（基于 tokio），且 0.5 与本 workspace 的
//! `async-trait` 直接兼容。本 crate 提供的 `RocketContext` 是
//! 框架无关的 stub，具体 `FromRequest` 实现请在用户项目中编写（约 30 行）。

use std::sync::Arc;

use sa_token_core::context::model::sa_request::SaRequest;
use sa_token_core::context::model::sa_response::SaResponse;
use sa_token_core::context::model::sa_storage::SaStorage;
use sa_token_core::context::sa_token_context::SaTokenContext;
use sa_token_core::context::sa_token_context_for_thread_local_staff::SaTokenContextForThreadLocalStaff;
use sa_token_core::exception::SaResult;
use sa_token_core::sa_manager::SaManager;
use sa_token_core::stp::stp_util::StpUtil;

/// Rocket 框架的 `SaTokenContext` 实现（stub，由用户在项目中重写以适配 Rocket Request）
#[derive(Clone, Default)]
pub struct RocketContext;

impl RocketContext {
    pub fn new() -> Self {
        Self
    }
}

impl SaTokenContext for RocketContext {
    fn set_context(
        &self,
        _req: Arc<dyn SaRequest>,
        _res: Arc<dyn SaResponse>,
        _stg: Arc<dyn SaStorage>,
    ) {
        // 默认 stub
    }

    fn clear_context(&self) {
        SaTokenContextForThreadLocalStaff::clear_model_box();
    }

    fn is_valid(&self) -> bool {
        SaTokenContextForThreadLocalStaff::get_model_box_or_null().is_some()
    }

    fn model_box(
        &self,
    ) -> sa_token_core::context::model::sa_token_context_model_box::SaTokenContextModelBox {
        SaTokenContextForThreadLocalStaff::get_model_box()
    }
}

/// 提取当前请求的 login_id（未登录则抛 NotLoginException）
pub fn extract_login_id() -> SaResult<String> {
    StpUtil::get_login_id()
}

/// 提取当前请求的 login_id（未登录返回 None）
pub fn extract_optional_login_id() -> SaResult<Option<String>> {
    StpUtil::get_login_id_default_null()
}

/// 注册一个全局 RocketContext
pub fn install_default_context() {
    SaManager::set_sa_token_context(Arc::new(RocketContext::new()));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rocket_context_default_construction() {
        let _ctx = RocketContext::new();
    }

    #[test]
    fn install_default_context_does_not_panic() {
        install_default_context();
    }
}
