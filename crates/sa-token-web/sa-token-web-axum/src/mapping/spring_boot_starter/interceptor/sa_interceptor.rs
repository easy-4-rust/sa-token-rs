//! Web integration mapping for Java `SaInterceptor`.
//! Responsibility is implemented by the `axum` adapter instead of Spring/Servlet crates.
pub use crate::auth_layer::{RequirePermissionLayer, RequireRoleLayer};
