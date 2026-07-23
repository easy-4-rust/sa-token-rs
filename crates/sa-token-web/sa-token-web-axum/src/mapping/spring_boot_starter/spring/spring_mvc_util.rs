//! Web integration mapping for Java `SpringMVCUtil`.
//! Responsibility is implemented by the `axum` adapter instead of Spring/Servlet crates.
/// Path matching is delegated to Axum/Tower route tables instead of Spring MVC helpers.
pub fn normalize_route(path: &str) -> String {
    path.trim_end_matches('/').to_string()
}
