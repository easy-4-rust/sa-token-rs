//! Web integration mapping for Java `SaApiKeyBeanRegister`.
//! Responsibility is implemented by the `salvo` adapter instead of Spring/Servlet crates.
use std::sync::Arc;
use sa_token_core::stp::AsyncStpUtil;

/// Salvo router wiring helper replacing Spring bean registration.
pub fn register_util(_router: &mut salvo::Router, util: Arc<AsyncStpUtil>) {
    let _ = util;
}
