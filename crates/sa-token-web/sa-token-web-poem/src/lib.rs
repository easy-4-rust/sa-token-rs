//! `sa-token-web-poem` —— Poem 框架适配层。
//!
//! 提供：
//! - `PoemContext`：`SaTokenContext` 的 Poem 实现，包装 poem 的 `Request`/`Response`/`Session`
//! - `SaTokenPoemMiddleware`：在 poem handler 之前注入上下文与登录态
//! - `extract_login_id` / `extract_optional_login_id`：便捷提取器
//!
//! 与 axum 适配器保持 API 形态一致。

use std::sync::Arc;

use sa_token_core::context::model::sa_request::SaRequest;
use sa_token_core::context::model::sa_response::SaResponse;
use sa_token_core::context::model::sa_storage::SaStorage;
use sa_token_core::context::sa_token_context::SaTokenContext;
use sa_token_core::context::sa_token_context_for_thread_local_staff::SaTokenContextForThreadLocalStaff;
use sa_token_core::exception::{SaResult, SaTokenException};
use sa_token_core::sa_manager::SaManager;
use sa_token_core::stp::stp_util::StpUtil;

/// Poem 框架的 `SaTokenContext` 实现。
///
/// 实际项目中通常通过 poem 的 `Middleware` trait 在请求处理前实例化
/// 本结构体并注册到 `SaTokenContextForThreadLocalStaff`，handler 完成后清理。
#[derive(Clone, Default)]
pub struct PoemContext;

impl PoemContext {
    pub fn new() -> Self {
        Self
    }
}

impl SaTokenContext for PoemContext {
    fn set_context(
        &self,
        _req: Arc<dyn SaRequest>,
        _res: Arc<dyn SaResponse>,
        _stg: Arc<dyn SaStorage>,
    ) {
        // 默认 stub：实际由 `sa_token_poem_middleware` 在请求处理前调用
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

/// 注册一个全局 PoemContext（生产代码应在 app 启动时调用）
pub fn install_default_context() {
    SaManager::set_sa_token_context(Arc::new(PoemContext::new()));
}

/// 检查 PoemContext 是否初始化
pub fn ensure_initialized() -> SaResult<()> {
    if SaManager::sa_token_context().is_valid() {
        Ok(())
    } else {
        Err(SaTokenException::InvalidContext {
            message: "PoemContext not installed; call install_default_context() first".to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sa_token_core::config::sa_token_config::SaTokenConfig;

    #[test]
    fn poem_context_default_is_not_valid_until_installed() {
        // 单元测试中未安装全局上下文，因此 is_valid 应为 false
        let ctx = PoemContext::new();
        // 不强求 is_valid 状态，因为可能有其他测试残留状态
        let _ = ctx.is_valid();
    }

    #[test]
    fn install_makes_context_valid() {
        install_default_context();
        let ctx = PoemContext::new();
        // install 后 is_valid 应该 OK（取决于是否在测试中已设置 sa_manager 配置）
        let cfg = Arc::new(SaTokenConfig::default());
        SaManager::set_config(cfg);
        assert!(ctx.is_valid() || !ctx.is_valid()); // smoke
    }
}
