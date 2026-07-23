//! Web integration mapping for Java `SaRequestForServlet`.
//! Responsibility is implemented by the `axum` adapter instead of Spring/Servlet crates.
pub use crate::request::AxumRequest as SaRequestForServlet;
