//! Web integration mapping for Java `SaStorageForServlet`.
//! Responsibility is implemented by the `axum` adapter instead of Spring/Servlet crates.
pub use crate::storage::AxumStorage as SaStorageForServlet;
