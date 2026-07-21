//! ThreadLocal 上下文实现（对应 Java `SaTokenContextForThreadLocal`）。
use std::cell::RefCell;
use std::sync::Arc;

use super::{
    model::{sa_request::SaRequest, sa_response::SaResponse, sa_storage::SaStorage},
    sa_token_context::SaTokenContext,
};

thread_local! {
    static CURRENT_REQ: RefCell<Option<Arc<dyn SaRequest>>> = const { RefCell::new(None) };
    static CURRENT_RES: RefCell<Option<Arc<dyn SaResponse>>> = const { RefCell::new(None) };
    static CURRENT_STG: RefCell<Option<Arc<dyn SaStorage>>> = const { RefCell::new(None) };
}

/// 基于 thread_local 的上下文实现
pub struct SaTokenContextForThreadLocal;

impl SaTokenContext for SaTokenContextForThreadLocal {
    fn set_context(
        &self,
        req: Arc<dyn SaRequest>,
        res: Arc<dyn SaResponse>,
        stg: Arc<dyn SaStorage>,
    ) {
        CURRENT_REQ.with(|cell| *cell.borrow_mut() = Some(req));
        CURRENT_RES.with(|cell| *cell.borrow_mut() = Some(res));
        CURRENT_STG.with(|cell| *cell.borrow_mut() = Some(stg));
    }

    fn clear_context(&self) {
        CURRENT_REQ.with(|cell| *cell.borrow_mut() = None);
        CURRENT_RES.with(|cell| *cell.borrow_mut() = None);
        CURRENT_STG.with(|cell| *cell.borrow_mut() = None);
    }

    fn is_valid(&self) -> bool {
        CURRENT_REQ.with(|cell| cell.borrow().is_some())
    }

    fn request(&self) -> Arc<dyn SaRequest> {
        CURRENT_REQ.with(|cell| {
            cell.borrow()
                .clone()
                .expect("No SaRequest available in thread-local context")
        })
    }

    fn response(&self) -> Arc<dyn SaResponse> {
        CURRENT_RES.with(|cell| {
            cell.borrow()
                .clone()
                .expect("No SaResponse available in thread-local context")
        })
    }

    fn storage(&self) -> Arc<dyn SaStorage> {
        CURRENT_STG.with(|cell| {
            cell.borrow()
                .clone()
                .expect("No SaStorage available in thread-local context")
        })
    }
}
