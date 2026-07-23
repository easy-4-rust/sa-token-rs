//! Web integration mapping for Java `SaResponseForReactor`.
//! Responsibility is implemented by the `actix` adapter instead of Spring/Servlet crates.
pub use actix_web::HttpResponseBuilder as SaResponseForReactor;
