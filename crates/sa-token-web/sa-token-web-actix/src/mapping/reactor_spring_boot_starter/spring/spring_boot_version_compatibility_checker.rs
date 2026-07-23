//! Web integration mapping for Java `SpringBootVersionCompatibilityChecker`.
//! Responsibility is implemented by the `actix` adapter instead of Spring/Servlet crates.
/// Spring Boot compatibility checks are not applicable in the Rust adapter stack.
pub fn assert_runtime_compatible() {}
