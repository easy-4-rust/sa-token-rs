//! Web integration mapping for Java `SaStorageForReactor`.
//! Responsibility is implemented by the `actix` adapter instead of Spring/Servlet crates.
pub use actix_web::web::Data as SaStorageForReactor;
