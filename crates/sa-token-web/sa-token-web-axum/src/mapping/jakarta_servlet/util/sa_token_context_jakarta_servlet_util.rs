//! Web integration mapping for Java `SaTokenContextJakartaServletUtil`.
//! Responsibility is implemented by the `axum` adapter instead of Spring/Servlet crates.
pub use crate::context::AxumContext as SaTokenContextForSpring;

/// Installs an Axum-backed Sa-Token context for the current request.
pub fn set_context(context: std::sync::Arc<crate::context::AxumContext>) {
    sa_token_core::sa_manager::SaManager::set_sa_token_context(context);
}
