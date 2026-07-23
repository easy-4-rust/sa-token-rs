//! Web integration mapping for Java `SaServletFilter`.
//! Responsibility is implemented by the `axum` adapter instead of Spring/Servlet crates.
pub use crate::layer::SaTokenLayer as SaServletFilter;
