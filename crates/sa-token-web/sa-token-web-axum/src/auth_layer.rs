//! Permission and role guard layers for Axum routes.

use axum::extract::Request;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use sa_token_core::stp::stp_util::StpUtil;

/// 权限检查 Extractor
pub struct RequirePermission;

impl RequirePermission {
    /// 创建权限检查层
    pub fn layer(permission: impl Into<String>) -> RequirePermissionLayer {
        RequirePermissionLayer {
            permission: permission.into(),
        }
    }
}

/// 权限检查层
pub struct RequirePermissionLayer {
    permission: String,
}

impl<S> tower::Layer<S> for RequirePermissionLayer {
    type Service = RequirePermissionService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        RequirePermissionService {
            inner,
            permission: self.permission.clone(),
        }
    }
}

/// 权限检查服务
#[derive(Clone)]
pub struct RequirePermissionService<S> {
    inner: S,
    permission: String,
}

impl<S> tower::Service<Request> for RequirePermissionService<S>
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
        let permission = self.permission.clone();

        Box::pin(async move {
            match StpUtil::check_permission(&permission) {
                Ok(_) => inner.call(req).await,
                Err(e) => Ok((StatusCode::FORBIDDEN, e.to_string()).into_response()),
            }
        })
    }
}

/// 角色检查 Extractor
pub struct RequireRole;

impl RequireRole {
    /// 创建角色检查层
    pub fn layer(role: impl Into<String>) -> RequireRoleLayer {
        RequireRoleLayer { role: role.into() }
    }
}

/// 角色检查层
pub struct RequireRoleLayer {
    role: String,
}

impl<S> tower::Layer<S> for RequireRoleLayer {
    type Service = RequireRoleService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        RequireRoleService {
            inner,
            role: self.role.clone(),
        }
    }
}

/// 角色检查服务
#[derive(Clone)]
pub struct RequireRoleService<S> {
    inner: S,
    role: String,
}

impl<S> tower::Service<Request> for RequireRoleService<S>
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
        let role = self.role.clone();

        Box::pin(async move {
            match StpUtil::check_role(&role) {
                Ok(_) => inner.call(req).await,
                Err(e) => Ok((StatusCode::FORBIDDEN, e.to_string()).into_response()),
            }
        })
    }
}
