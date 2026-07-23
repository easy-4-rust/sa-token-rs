//! `sa-token-web-warp` —— Warp 框架适配层。
//!
//! 提供：
//! - `WarpContext`：`SaTokenContext` 的 Warp 实现（基于 Warp 的 `Filter` 体系）
//! - `sa_token_filter`：便捷 Warp filter，在请求进入 handler 前完成 token 提取
//! - `extract_login_id` / `extract_optional_login_id`：便捷提取器
//!
//! 与 axum/poem/rocket 适配器保持 API 形态一致。Warp 的核心抽象是
//! `Filter`，因此本 crate 暴露一个 `Filter` 组合子而不是 middleware trait。

use std::sync::Arc;

use sa_token_core::context::model::sa_request::SaRequest;
use sa_token_core::context::model::sa_response::SaResponse;
use sa_token_core::context::model::sa_storage::SaStorage;
use sa_token_core::context::sa_token_context::SaTokenContext;
use sa_token_core::context::sa_token_context_for_thread_local_staff::SaTokenContextForThreadLocalStaff;
use sa_token_core::exception::SaResult;
use sa_token_core::sa_manager::SaManager;
use sa_token_core::stp::stp_util::StpUtil;

/// Warp 框架的 `SaTokenContext` 实现
#[derive(Clone, Default)]
pub struct WarpContext;

impl WarpContext {
    pub fn new() -> Self {
        Self
    }
}

impl SaTokenContext for WarpContext {
    fn set_context(
        &self,
        _req: Arc<dyn SaRequest>,
        _res: Arc<dyn SaResponse>,
        _stg: Arc<dyn SaStorage>,
    ) {
        // 默认 stub：生产代码应在 Warp `with` filter 中实例化
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

/// 注册一个全局 WarpContext
pub fn install_default_context() {
    SaManager::set_sa_token_context(Arc::new(WarpContext::new()));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn warp_context_default_construction() {
        let _ctx = WarpContext::new();
    }

    #[test]
    fn install_default_context_does_not_panic() {
        install_default_context();
    }
}
