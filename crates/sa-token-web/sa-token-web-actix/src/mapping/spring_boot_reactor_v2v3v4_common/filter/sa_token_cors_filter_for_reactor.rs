//! Web integration mapping for Java `SaTokenCorsFilterForReactor`.
//! Responsibility is implemented by the `actix` adapter instead of Spring/Servlet crates.
/// CORS is configured at the framework layer in Actix/Salvo apps.
pub fn cors_is_framework_managed() -> bool { true }
