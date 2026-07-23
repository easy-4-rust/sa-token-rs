//! Web integration mapping for Java `SaReactorFilter`.
//! Responsibility is implemented by the `actix` adapter instead of Spring/Servlet crates.
pub use crate::middleware::require_login as sa_reactor_filter;
