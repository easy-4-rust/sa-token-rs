//! 上下文门面（对应 Java `cn.dev33.satoken.context.SaHolder`）。
use std::sync::Arc;

use super::{
    model::{sa_request::SaRequest, sa_response::SaResponse, sa_storage::SaStorage},
    sa_token_context::SaTokenContext,
};
use crate::sa_manager::SaManager;

/// 上下文门面
///
/// 提供便捷方法获取当前请求的上下文对象。
pub struct SaHolder;

impl SaHolder {
    /// 获取当前上下文
    pub fn get_context() -> Arc<dyn SaTokenContext> {
        SaManager::sa_token_context()
    }

    /// 获取当前请求
    pub fn get_request() -> Arc<dyn SaRequest> {
        Self::get_context().request()
    }

    /// 获取当前响应
    pub fn get_response() -> Arc<dyn SaResponse> {
        Self::get_context().response()
    }

    /// 获取当前存储
    pub fn get_storage() -> Arc<dyn SaStorage> {
        Self::get_context().storage()
    }
}
