//! Web integration mapping for Java `SaTokenContextForSpringReactor`.
//! Responsibility is implemented by the `actix` adapter instead of Spring/Servlet crates.
pub use crate::runtime_wiring::register_async_runtime;

/// Reactor/WebFlux context registration maps to explicit Actix app data wiring.
pub fn set_context(_: ()) {}
