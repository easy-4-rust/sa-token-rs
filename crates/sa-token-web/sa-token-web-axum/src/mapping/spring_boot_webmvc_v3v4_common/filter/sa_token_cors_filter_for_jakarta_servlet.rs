//! Web integration mapping for Java `SaTokenCorsFilterForJakartaServlet`.
//! Responsibility is implemented by the `axum` adapter instead of Spring/Servlet crates.
pub use tower_http::cors::{Any, CorsLayer};
