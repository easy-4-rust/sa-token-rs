//! `SaTokenContextModelBox` —— 1:1 对应 Java `cn.dev33.satoken.context.model.SaTokenContextModelBox`
//!
//! 用于在线程或上下文中持有 request/response/storage 三个包装对象。

use std::sync::Arc;

use super::{sa_request::SaRequest, sa_response::SaResponse, sa_storage::SaStorage};

/// 包装 request、response、storage 的容器
#[derive(Clone)]
pub struct SaTokenContextModelBox {
    /// Request 包装对象
    pub request: Arc<dyn SaRequest>,
    /// Response 包装对象
    pub response: Arc<dyn SaResponse>,
    /// Storage 包装对象
    pub storage: Arc<dyn SaStorage>,
}

impl SaTokenContextModelBox {
    /// 构造 ModelBox
    pub fn new(
        request: Arc<dyn SaRequest>,
        response: Arc<dyn SaResponse>,
        storage: Arc<dyn SaStorage>,
    ) -> Self {
        Self {
            request,
            response,
            storage,
        }
    }

    /// 获取 Request（对应 Java `getRequest()`）
    pub fn get_request(&self) -> Arc<dyn SaRequest> {
        self.request.clone()
    }

    /// 获取 Response（对应 Java `getResponse()`）
    pub fn get_response(&self) -> Arc<dyn SaResponse> {
        self.response.clone()
    }

    /// 获取 Storage（对应 Java `getStorage()`）
    pub fn get_storage(&self) -> Arc<dyn SaStorage> {
        self.storage.clone()
    }
}
