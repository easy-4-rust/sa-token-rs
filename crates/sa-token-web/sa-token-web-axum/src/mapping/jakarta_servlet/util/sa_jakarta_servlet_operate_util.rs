//! Web integration mapping for Java `SaJakartaServletOperateUtil`.
//! Responsibility is implemented by the `axum` adapter instead of Spring/Servlet crates.
pub use crate::token::{extract_token_from_headers, extract_token_from_request_parts};

/// Reads the configured token from an Axum request snapshot.
pub fn read_token(token_name: &str, headers: &[(String, String)], cookies: &[(String, String)]) -> Option<String> {
    extract_token_from_headers(token_name, headers, cookies)
}
