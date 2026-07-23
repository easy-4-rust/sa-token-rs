//! Sa-Token Axum middleware layer.

use std::sync::Arc;

use axum::extract::Request;
use axum::response::Response;
use sa_token_core::sa_manager::SaManager;

use crate::context::AxumContext;
use crate::request::AxumRequest;

/// Sa-Token Axum 中间件层
///
/// 自动将请求注入到 Sa-Token 上下文中。
#[derive(Clone)]
pub struct SaTokenLayer;

impl SaTokenLayer {
    /// 创建新的 SaTokenLayer
    pub fn new() -> Self {
        Self
    }
}

impl Default for SaTokenLayer {
    fn default() -> Self {
        Self::new()
    }
}

impl<S> tower::Layer<S> for SaTokenLayer {
    type Service = SaTokenService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        SaTokenService { inner }
    }
}

/// Sa-Token 中间件服务
#[derive(Clone)]
pub struct SaTokenService<S> {
    inner: S,
}

impl<S> tower::Service<Request> for SaTokenService<S>
where
    S: tower::Service<Request, Response = Response> + Send + Clone + 'static,
    S::Future: Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = futures_util::future::BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request) -> Self::Future {
        let mut inner = self.inner.clone();

        Box::pin(async move {
            let axum_req = AxumRequest::from_axum_request(&req);
            let ctx = AxumContext::new(axum_req);
            SaManager::set_sa_token_context(Arc::new(ctx));
            inner.call(req).await
        })
    }
}
