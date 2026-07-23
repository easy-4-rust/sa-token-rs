//! ThreadLocal 上下文实现（对应 Java `SaTokenContextForThreadLocal`）。
use std::sync::Arc;

use super::{
    model::{
        sa_request::SaRequest, sa_response::SaResponse, sa_storage::SaStorage,
        sa_token_context_model_box::SaTokenContextModelBox,
    },
    sa_token_context::SaTokenContext,
    sa_token_context_for_thread_local_staff::SaTokenContextForThreadLocalStaff,
};

/// 基于 ThreadLocal 的上下文实现
pub struct SaTokenContextForThreadLocal;

impl SaTokenContext for SaTokenContextForThreadLocal {
    fn set_context(
        &self,
        req: Arc<dyn SaRequest>,
        res: Arc<dyn SaResponse>,
        stg: Arc<dyn SaStorage>,
    ) {
        SaTokenContextForThreadLocalStaff::set_model_box(req, res, stg);
    }

    fn clear_context(&self) {
        SaTokenContextForThreadLocalStaff::clear_model_box();
    }

    fn is_valid(&self) -> bool {
        SaTokenContextForThreadLocalStaff::get_model_box_or_null().is_some()
    }

    fn model_box(&self) -> SaTokenContextModelBox {
        SaTokenContextForThreadLocalStaff::get_model_box()
    }
}
