//! 路由拦截器函数端口。

use std::any::Any;

use crate::context::model::{sa_request::SaRequest, sa_response::SaResponse};

/// Java `SaRouteFunction` 的 Rust 对应 trait。
pub trait SaRouteFunction: Send + Sync {
    /// 使用请求、响应和框架 handler 执行验证。
    fn run(&self, request: &dyn SaRequest, response: &dyn SaResponse, handler: &dyn Any);
}

impl<F> SaRouteFunction for F
where
    F: Fn(&dyn SaRequest, &dyn SaResponse, &dyn Any) + Send + Sync,
{
    fn run(&self, request: &dyn SaRequest, response: &dyn SaResponse, handler: &dyn Any) {
        self(request, response, handler);
    }
}
