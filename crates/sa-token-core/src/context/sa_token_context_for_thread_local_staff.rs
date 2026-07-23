//! `SaTokenContextForThreadLocalStaff` —— 1:1 对应 Java `cn.dev33.satoken.context.SaTokenContextForThreadLocalStaff`
//!
//! 基于 ThreadLocal 的 Box 存储器，配合 `SaTokenContextForThreadLocal` 使用。

use std::cell::RefCell;
use std::sync::Arc;

use super::{
    model::{
        sa_request::SaRequest, sa_response::SaResponse, sa_storage::SaStorage,
        sa_token_context_model_box::SaTokenContextModelBox,
    },
    sa_token_context_error::raise_context_not_initialized,
};

thread_local! {
    static MODEL_BOX: RefCell<Option<SaTokenContextModelBox>> = const { RefCell::new(None) };
}

/// ThreadLocal Box 存储器（对应 Java 静态工具类）
pub struct SaTokenContextForThreadLocalStaff;

impl SaTokenContextForThreadLocalStaff {
    /// 初始化当前线程的 Box（对应 Java `setModelBox`）
    pub fn set_model_box(
        request: Arc<dyn SaRequest>,
        response: Arc<dyn SaResponse>,
        storage: Arc<dyn SaStorage>,
    ) {
        MODEL_BOX.with(|cell| {
            *cell.borrow_mut() = Some(SaTokenContextModelBox::new(request, response, storage));
        });
    }

    /// 清除当前线程的 Box（对应 Java `clearModelBox`）
    pub fn clear_model_box() {
        MODEL_BOX.with(|cell| *cell.borrow_mut() = None);
    }

    /// 获取 Box，可能为空（对应 Java `getModelBoxOrNull`）
    pub fn get_model_box_or_null() -> Option<SaTokenContextModelBox> {
        MODEL_BOX.with(|cell| cell.borrow().clone())
    }

    /// 获取 Box，为空则抛出上下文异常（对应 Java `getModelBox`）
    pub fn get_model_box() -> SaTokenContextModelBox {
        Self::get_model_box_or_null().unwrap_or_else(|| raise_context_not_initialized())
    }

    /// 获取当前线程 Request（对应 Java `getRequest`）
    pub fn get_request() -> Arc<dyn SaRequest> {
        Self::get_model_box().get_request()
    }

    /// 获取当前线程 Response（对应 Java `getResponse`）
    pub fn get_response() -> Arc<dyn SaResponse> {
        Self::get_model_box().get_response()
    }

    /// 获取当前线程 Storage（对应 Java `getStorage`）
    pub fn get_storage() -> Arc<dyn SaStorage> {
        Self::get_model_box().get_storage()
    }

    /// 内部测试钩子：在未初始化时触发与 DefaultImpl 相同的处理器错误
    #[cfg(test)]
    pub fn require_handler_or_raise() -> ! {
        use super::sa_token_context_error::raise_invalid_context_handler;
        raise_invalid_context_handler();
    }
}
