//! Web integration mapping for Java `SaServletErrorCode`.
//! Responsibility is implemented by the `axum` adapter instead of Spring/Servlet crates.
pub use sa_token_core::error::sa_error_code::SaErrorCode as SaServletErrorCode;
