//! 默认上下文实现（对应 Java `SaTokenContextDefaultImpl`）。
use std::sync::Arc;

use super::{
    model::{sa_request::SaRequest, sa_response::SaResponse, sa_storage::SaStorage},
    sa_token_context::SaTokenContext,
};

/// 默认上下文实现（不做任何操作，用于非 Web 环境）
pub struct SaTokenContextDefaultImpl;

impl SaTokenContext for SaTokenContextDefaultImpl {
    fn set_context(
        &self,
        _req: Arc<dyn SaRequest>,
        _res: Arc<dyn SaResponse>,
        _stg: Arc<dyn SaStorage>,
    ) {
        // 默认实现不存储上下文
    }

    fn clear_context(&self) {
        // 默认实现不存储上下文
    }

    fn is_valid(&self) -> bool {
        false
    }

    fn request(&self) -> Arc<dyn SaRequest> {
        panic!("No SaRequest available in default context")
    }

    fn response(&self) -> Arc<dyn SaResponse> {
        panic!("No SaResponse available in default context")
    }

    fn storage(&self) -> Arc<dyn SaStorage> {
        panic!("No SaStorage available in default context")
    }
}
