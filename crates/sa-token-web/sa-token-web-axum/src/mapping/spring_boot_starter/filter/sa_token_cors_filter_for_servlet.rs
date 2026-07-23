//! Web integration mapping for Java `SaTokenCorsFilterForServlet`.
//! Responsibility is implemented by the `axum` adapter instead of Spring/Servlet crates.
pub use tower_http::cors::{Any, CorsLayer};
