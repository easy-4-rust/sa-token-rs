//! Web integration mapping for Java `SaPathMatcherHolder`.
//! Responsibility is implemented by the `salvo` adapter instead of Spring/Servlet crates.
/// Path matching is delegated to Axum/Tower route tables instead of Spring MVC helpers.
pub fn normalize_route(path: &str) -> String {
    path.trim_end_matches('/').to_string()
}
