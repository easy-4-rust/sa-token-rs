//! Web integration mapping for Java `SaResponseForServlet`.
//! Responsibility is implemented by the `axum` adapter instead of Spring/Servlet crates.
pub use crate::response::AxumResponse as SaResponseForServlet;
