//! `SaTokenContextMockUtil` —— 1:1 对应 Java `cn.dev33.satoken.context.mock.SaTokenContextMockUtil`

use std::sync::Arc;

use super::super::model::{sa_request::SaRequest, sa_response::SaResponse, sa_storage::SaStorage};
use super::sa_request_for_mock::SaRequestForMock;
use super::sa_response_for_mock::SaResponseForMock;
use super::sa_storage_for_mock::SaStorageForMock;
use crate::sa_manager::SaManager;

/// Mock 上下文工具（对应 Java 静态工具类）
pub struct SaTokenContextMockUtil;

impl SaTokenContextMockUtil {
    /// 写入 Mock 上下文（对应 Java `setMockContext()`）
    pub fn set_mock_context() {
        let request: Arc<dyn SaRequest> = Arc::new(SaRequestForMock::new());
        let response: Arc<dyn SaResponse> = Arc::new(SaResponseForMock::new());
        let storage: Arc<dyn SaStorage> = Arc::new(SaStorageForMock::new());
        SaManager::sa_token_context().set_context(request, response, storage);
    }

    /// 写入 Mock 上下文并执行回调，结束后自动清除（对应 Java `setMockContext(SaFunction)`）
    pub fn set_mock_context_fn<F>(fun: F)
    where
        F: FnOnce(),
    {
        Self::set_mock_context();
        struct Guard;
        impl Drop for Guard {
            fn drop(&mut self) {
                SaTokenContextMockUtil::clear_context();
            }
        }
        let _guard = Guard;
        fun();
    }

    /// 写入 Mock 上下文并执行回调，结束后自动清除（对应 Java `setMockContext(SaRetGenericFunction)`）
    pub fn set_mock_context_with<F, R>(fun: F) -> R
    where
        F: FnOnce() -> R,
    {
        Self::set_mock_context();
        struct Guard;
        impl Drop for Guard {
            fn drop(&mut self) {
                SaTokenContextMockUtil::clear_context();
            }
        }
        let _guard = Guard;
        fun()
    }

    /// 清除上下文（对应 Java `clearContext()`）
    pub fn clear_context() {
        SaManager::sa_token_context().clear_context();
    }

    /// 创建一组 mock 的 request/response/storage（测试辅助）
    pub fn create_mock_context(
        url: &str,
        method: &str,
    ) -> (Arc<dyn SaRequest>, Arc<dyn SaResponse>, Arc<dyn SaStorage>) {
        let req: Arc<dyn SaRequest> =
            Arc::new(SaRequestForMock::new().with_url(url).with_method(method));
        let res: Arc<dyn SaResponse> = Arc::new(SaResponseForMock::new());
        let stg: Arc<dyn SaStorage> = Arc::new(SaStorageForMock::new());
        (req, res, stg)
    }
}
