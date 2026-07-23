//! Sa-Token context binding for Axum requests.

use std::sync::Arc;

use sa_token_core::context::model::sa_request::SaRequest;
use sa_token_core::context::model::sa_response::SaResponse;
use sa_token_core::context::model::sa_storage::SaStorage;
use sa_token_core::context::model::sa_token_context_model_box::SaTokenContextModelBox;
use sa_token_core::context::sa_token_context::SaTokenContext;

use crate::request::AxumRequest;
use crate::response::AxumResponse;
use crate::storage::AxumStorage;

/// Axum 上下文实现
pub struct AxumContext {
    /// 请求
    request: Arc<AxumRequest>,
    /// 响应
    response: Arc<AxumResponse>,
    /// 存储
    storage: Arc<AxumStorage>,
}

impl AxumContext {
    /// 创建新的 Axum 上下文
    pub fn new(request: AxumRequest) -> Self {
        Self {
            request: Arc::new(request),
            response: Arc::new(AxumResponse::default()),
            storage: Arc::new(AxumStorage::default()),
        }
    }
}

impl SaTokenContext for AxumContext {
    fn set_context(
        &self,
        _req: Arc<dyn SaRequest>,
        _res: Arc<dyn SaResponse>,
        _stg: Arc<dyn SaStorage>,
    ) {
        // Axum 上下文在创建时已设置
    }

    fn clear_context(&self) {
        self.storage.clear();
    }

    fn is_valid(&self) -> bool {
        true
    }

    fn model_box(&self) -> SaTokenContextModelBox {
        SaTokenContextModelBox::new(
            self.request.clone(),
            self.response.clone(),
            self.storage.clone(),
        )
    }
}
