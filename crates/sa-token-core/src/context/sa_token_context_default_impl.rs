//! 默认上下文实现（对应 Java `SaTokenContextDefaultImpl`）。
use std::sync::Arc;

use super::{
    model::{
        sa_request::SaRequest, sa_response::SaResponse, sa_storage::SaStorage,
        sa_token_context_model_box::SaTokenContextModelBox,
    },
    sa_token_context::SaTokenContext,
    sa_token_context_error::raise_invalid_context_handler,
};

/// 错误提示语（对应 Java `ERROR_MESSAGE`）
pub const ERROR_MESSAGE: &str = "未能获取有效的上下文处理器";

/// 默认上下文实现
pub struct SaTokenContextDefaultImpl;

/// 默认实例（对应 Java `defaultContext`）
pub static DEFAULT_CONTEXT: SaTokenContextDefaultImpl = SaTokenContextDefaultImpl;

impl SaTokenContext for SaTokenContextDefaultImpl {
    fn set_context(
        &self,
        _req: Arc<dyn SaRequest>,
        _res: Arc<dyn SaResponse>,
        _stg: Arc<dyn SaStorage>,
    ) {
        raise_invalid_context_handler();
    }

    fn clear_context(&self) {
        raise_invalid_context_handler();
    }

    fn is_valid(&self) -> bool {
        raise_invalid_context_handler();
    }

    fn model_box(&self) -> SaTokenContextModelBox {
        raise_invalid_context_handler();
    }
}
