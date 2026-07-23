//! `sa-token-rpc-tarpc` —— tarpc (Google Rust RPC) 中间件。
//!
//! tarpc 0.34 的 `Context` 不再提供通用的 key-value extra（迁移到
//! `tracing::Span`）。本 crate 用 `tokio::task_local!` 传递 sa-token token，
//! 与 axum/tonic 适配器保持 API 一致。
//!
//! 用法（在用户项目中）：
//! ```ignore
//! use sa_token_rpc_tarpc::{RPC_TOKEN, check_login_from_scope};
//! RPC_TOKEN.scope("tok-abc".to_string(), async {
//!     let login_id = check_login_from_scope().await.unwrap();
//!     // handler logic
//! }).await;
//! ```

use std::cell::RefCell;

use sa_token_core::exception::SaResult;
use sa_token_core::stp::stp_util::StpUtil;

/// tarpc 0.34 不再支持 `Context::insert`，因此用 `tokio::task_local!`
/// 传递 token。生产代码需要在 handler 入口处用 `RPC_TOKEN.scope(token, ...)`
/// 包裹整个请求处理。
tokio::task_local! {
    /// 当前请求关联的 sa-token token
    pub static RPC_TOKEN: String;
}

/// 线程本地的兜底（同步代码路径或测试）
thread_local! {
    static TLS_TOKEN: RefCell<Option<String>> = const { RefCell::new(None) };
}

/// 设置当前线程的 sa-token token（用于非 async 场景或测试）
pub fn set_thread_local_token(token: Option<String>) {
    TLS_TOKEN.with(|t| *t.borrow_mut() = token);
}

/// 在 tokio task-local 作用域内执行闭包，并把 token 注入
pub async fn with_token<F, R>(token: impl Into<String>, f: F) -> R
where
    F: std::future::Future<Output = R>,
{
    let token = token.into();
    RPC_TOKEN.scope(token, f).await
}

/// 同步路径：从 thread-local 读取 token
pub fn get_token_sync() -> Option<String> {
    TLS_TOKEN.with(|t| t.borrow().clone())
}

/// 异步路径：从 tokio task-local 读取 token（带 fallback 到 thread-local）
pub fn get_token_in_task() -> Option<String> {
    // 由于 tokio::task_local! 只能在 scope 内访问，这里用 try_with
    match RPC_TOKEN.try_with(|t| t.clone()) {
        Ok(t) => Some(t),
        Err(_) => get_token_sync(),
    }
}

/// 校验 tarpc 请求中的 token，返回 login_id 或 None
pub fn check_login_from_scope() -> SaResult<Option<String>> {
    let token = match get_token_in_task() {
        None => return Ok(None),
        Some(t) => t,
    };
    let login_id = StpUtil::get_login_id_by_token(&token)?;
    Ok(login_id)
}

/// 校验 tarpc 请求中的 token，要求已登录，否则返回 NotLoginException
pub fn require_login_from_scope() -> SaResult<String> {
    check_login_from_scope()?.ok_or_else(|| {
        sa_token_core::exception::SaTokenException::NotLogin {
            message: "tarpc request missing sa-token".to_string(),
            login_type: "login".to_string(),
            scene: "NOT_TOKEN".to_string(),
            code: 0,
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use sa_token_core::config::sa_token_config::SaTokenConfig;
    use sa_token_core::dao::sa_token_dao_default_impl::SaTokenDaoDefaultImpl;
    use sa_token_core::sa_manager::SaManager;
    use std::sync::Arc;

    fn setup() {
        SaManager::reset();
        SaManager::set_config(Arc::new(SaTokenConfig::default()));
        SaManager::set_sa_token_dao(Arc::new(SaTokenDaoDefaultImpl::new()));
        set_thread_local_token(None);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn check_login_returns_none_without_token() {
        setup();
        let result = check_login_from_scope().expect("ok");
        assert!(result.is_none());
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn require_login_errors_without_token() {
        setup();
        let result = require_login_from_scope();
        assert!(result.is_err());
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn with_token_propagates_via_task_local() {
        setup();
        let result = with_token("test-tok", async {
            // task-local 应该返回注入的 token
            get_token_in_task()
        })
        .await;
        assert_eq!(result.as_deref(), Some("test-tok"));
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn thread_local_token_round_trip() {
        setup();
        set_thread_local_token(Some("tl-tok".to_string()));
        assert_eq!(get_token_sync().as_deref(), Some("tl-tok"));
    }
}
