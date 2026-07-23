//! Web integration mapping for Java `ApplicationContextPathLoading`.
//! Responsibility is implemented by the `salvo` adapter instead of Spring/Servlet crates.
/// Context-path prefixes are configured on the Axum `Router` nest path.
pub fn apply_context_path(base: &str, route: &str) -> String {
    format!("{}{}", base.trim_end_matches('/'), route)
}
