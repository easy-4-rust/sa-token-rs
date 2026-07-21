//! 上下文抽象（对应 Java `cn.dev33.satoken.context.SaTokenContext`）。
use std::sync::Arc;

use super::model::{sa_request::SaRequest, sa_response::SaResponse, sa_storage::SaStorage};

/// 上下文抽象 trait
///
/// 对应 Java `SaTokenContext`，用于隔离不同 Web 框架的请求/响应实现。
pub trait SaTokenContext: Send + Sync {
    /// 设置上下文
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

    /// 获取请求对象
    fn request(&self) -> Arc<dyn SaRequest>;

    /// 获取响应对象
    fn response(&self) -> Arc<dyn SaResponse>;

    /// 获取存储对象
    fn storage(&self) -> Arc<dyn SaStorage>;
}
