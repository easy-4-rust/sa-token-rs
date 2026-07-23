//! Web integration mapping for Java `SaRequestForReactor`.
//! Responsibility is implemented by the `actix` adapter instead of Spring/Servlet crates.
pub use actix_web::HttpRequest as SaRequestForReactor;
