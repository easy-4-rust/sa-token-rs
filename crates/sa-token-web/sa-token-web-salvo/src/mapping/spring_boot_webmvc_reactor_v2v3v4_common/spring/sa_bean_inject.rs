//! Web integration mapping for Java `SaBeanInject`.
//! Responsibility is implemented by the `salvo` adapter instead of Spring/Servlet crates.
use std::sync::Arc;
use sa_token_core::stp::AsyncStpUtil;

/// Dependency injection is explicit in Rust; callers pass `Arc<AsyncStpUtil>` into handlers.
pub fn inject_util(util: Arc<AsyncStpUtil>) -> Arc<AsyncStpUtil> {
    util
}
