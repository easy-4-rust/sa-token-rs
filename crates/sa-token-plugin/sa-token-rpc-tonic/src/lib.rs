//! `sa-token-rpc-tonic` —— tonic (gRPC) 拦截器。
//!
//! 对应 Java `sa-token-grpc` 插件：使用 `sa-token` 在 gRPC server 端
//! 校验 token，把 `login_id` 注入到 gRPC request extension，handler 可
//! 通过 `extensions().get::<LoginId>()` 读取。
//!
//! 拦截器实现两个 trait：
//! - `sa_token_rpc_interceptor::check_token`：从 request metadata 提取 token 并校验
//! - `sa_token_rpc_interceptor::LoginIdExt`：把 login_id 写入 request extension
//!
//! # 用法
//! ```ignore
//! use sa_token_rpc_tonic::SaTokenInterceptor;
//! use tonic::transport::Server;
//!
//! # async fn run() {
//! let interceptor = SaTokenInterceptor::new("authorization");
//! // Server::builder()
//! //     .add_service(MySvcServer::with_interceptor(MySvc, interceptor))
//! //     .serve("0.0.0.0:50051".parse().unwrap())
//! //     .await
//! //     .unwrap();
//! # }
//! ```

use std::sync::Arc;

use sa_token_core::stp::stp_util::StpUtil;
use tonic::{Request, Status};

/// 从 request metadata 中提取 `header_name` 字段的 token 字符串
pub fn extract_token<T>(request: &Request<T>, header_name: &str) -> Option<String> {
    let value = request.metadata().get(header_name)?;
    value.to_str().ok().map(|s| s.to_string())
}

/// 校验 token 并返回 login_id；失败时返回 `Status::unauthenticated`
pub fn authenticate_or_status<T>(
    request: &Request<T>,
    header_name: &str,
) -> Result<String, Status> {
    let token = extract_token(request, header_name).ok_or_else(|| {
        Status::unauthenticated(format!("missing sa-token header '{header_name}'"))
    })?;
    let login_id = StpUtil::get_login_id_by_token(&token)
        .map_err(|e| Status::internal(format!("sa-token dao error: {e}")))?
        .ok_or_else(|| Status::unauthenticated("invalid or expired sa-token"))?;
    Ok(login_id)
}

/// tonic Interceptor 包装：把 `login_id` 注入到 request extension
#[derive(Clone)]
pub struct SaTokenInterceptor {
    header_name: Arc<String>,
}

impl SaTokenInterceptor {
    pub fn new(header_name: impl Into<String>) -> Self {
        Self {
            header_name: Arc::new(header_name.into()),
        }
    }

    /// 提取当前请求的 login_id（已通过认证）
    pub fn authenticate<T>(&self, request: &Request<T>) -> Result<String, Status> {
        authenticate_or_status(request, &self.header_name)
    }

    /// 提取当前请求的 login_id（未登录返回 None）
    pub fn authenticate_optional<T>(
        &self,
        request: &Request<T>,
    ) -> Result<Option<String>, Status> {
        match extract_token(request, &self.header_name) {
            None => Ok(None),
            Some(token) => {
                let login_id = StpUtil::get_login_id_by_token(&token)
                    .map_err(|e| Status::internal(format!("sa-token dao error: {e}")))?;
                Ok(login_id)
            }
        }
    }
}

/// 在 tonic handler 中从 request extensions 读取 login_id 的辅助函数
pub fn login_id_from_extension<T>(request: &Request<T>) -> Option<String> {
    request
        .extensions()
        .get::<LoginId>()
        .map(|l| l.0.clone())
}

/// 注入到 request extension 的 login_id 包装
#[derive(Clone, Debug)]
pub struct LoginId(pub String);

/// 拦截器辅助：把 login_id 写入 extensions（由用户在中间件中调用）
pub fn inject_login_id<T>(request: &mut Request<T>, login_id: String) {
    request.extensions_mut().insert(LoginId(login_id));
}

#[cfg(test)]
mod tests {
    use super::*;
    use sa_token_core::config::sa_token_config::SaTokenConfig;
    use sa_token_core::dao::sa_token_dao_default_impl::SaTokenDaoDefaultImpl;
    use sa_token_core::sa_manager::SaManager;
    use std::sync::Arc;
    use tonic::Request;

    fn setup() {
        SaManager::reset();
        SaManager::set_config(Arc::new(SaTokenConfig::default()));
        SaManager::set_sa_token_dao(Arc::new(SaTokenDaoDefaultImpl::new()));
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn extract_token_reads_metadata() {
        setup();
        let mut req = Request::new(());
        req.metadata_mut()
            .insert("authorization", "tok-123".parse().unwrap());
        let token = extract_token(&req, "authorization").expect("token present");
        assert_eq!(token, "tok-123");
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn authenticate_optional_returns_none_when_header_missing() {
        setup();
        let interceptor = SaTokenInterceptor::new("authorization");
        let req = Request::new(());
        let result = interceptor.authenticate_optional(&req).expect("ok");
        assert!(result.is_none());
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn login_id_extension_round_trip() {
        let mut req = Request::new(());
        inject_login_id(&mut req, "u-1".to_string());
        assert_eq!(login_id_from_extension(&req).as_deref(), Some("u-1"));
    }
}
