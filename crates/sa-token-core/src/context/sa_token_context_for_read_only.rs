//! `SaTokenContextForReadOnly` —— 1:1 对应 Java `cn.dev33.satoken.context.SaTokenContextForReadOnly`

use std::sync::Arc;

use super::{
    model::{
        sa_request::SaRequest, sa_response::SaResponse, sa_storage::SaStorage,
        sa_token_context_model_box::SaTokenContextModelBox,
    },
    sa_token_context::SaTokenContext,
};

/// 只读上下文包装器
pub struct SaTokenContextForReadOnly {
    /// 内部持有的真实上下文
    pub inner: Box<dyn SaTokenContext>,
}

impl SaTokenContextForReadOnly {
    /// 包装已有上下文为只读
    pub fn new(inner: Box<dyn SaTokenContext>) -> Self {
        Self { inner }
    }
}

impl SaTokenContext for SaTokenContextForReadOnly {
    fn set_context(
        &self,
        _req: Arc<dyn SaRequest>,
        _res: Arc<dyn SaResponse>,
        _stg: Arc<dyn SaStorage>,
    ) {
        // 只读：忽略写入
    }

    fn clear_context(&self) {
        // 只读：忽略清除
    }

    fn is_valid(&self) -> bool {
        self.inner.is_valid()
    }

    fn model_box(&self) -> SaTokenContextModelBox {
        self.inner.model_box()
    }
}
