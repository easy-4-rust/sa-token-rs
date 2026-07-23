//! Web integration mapping for Java `SaTokenContextRegister`.
//! Responsibility is implemented by the `axum` adapter instead of Spring/Servlet crates.
use std::sync::Arc;
use sa_token_core::context::sa_token_context::SaTokenContext;

/// Axum state wiring helper replacing Spring bean registration.
pub fn register_context(context: Arc<dyn SaTokenContext>) {
    sa_token_core::sa_manager::SaManager::set_sa_token_context(context);
}
