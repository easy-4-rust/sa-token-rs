//! 上下文抽象（对应 Java `cn.dev33.satoken.context.SaTokenContext`）。
use std::sync::Arc;

use super::model::{
    sa_request::SaRequest, sa_response::SaResponse, sa_storage::SaStorage,
    sa_token_context_model_box::SaTokenContextModelBox,
};

/// 上下文抽象 trait
///
/// 对应 Java `SaTokenContext`，用于隔离不同 Web 框架的请求/响应实现。
pub trait SaTokenContext: Send + Sync {
    /// 初始化上下文
    fn set_context(
        &self,
        req: Arc<dyn SaRequest>,
        res: Arc<dyn SaResponse>,
        stg: Arc<dyn SaStorage>,
    );

    /// 清除上下文
    fn clear_context(&self);

    /// 上下文是否有效
    fn is_valid(&self) -> bool;

    /// 获取 Box 对象（对应 Java `getModelBox()`）
    fn model_box(&self) -> SaTokenContextModelBox;

    /// 获取当前请求（对应 Java 默认方法 `getRequest()`）
    fn request(&self) -> Arc<dyn SaRequest> {
        self.model_box().get_request()
    }

    /// 获取当前响应（对应 Java 默认方法 `getResponse()`）
    fn response(&self) -> Arc<dyn SaResponse> {
        self.model_box().get_response()
    }

    /// 获取当前存储（对应 Java 默认方法 `getStorage()`）
    fn storage(&self) -> Arc<dyn SaStorage> {
        self.model_box().get_storage()
    }
}
