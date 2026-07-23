//! Sa-Token Axum 适配层
//!
//! Servlet / Spring MVC responsibilities are mapped onto Axum middleware,
//! extractors, and context models instead of Spring starter crates.

mod auth_layer;
mod context;
mod extractors;
mod layer;
pub mod mapping;
mod request;
mod response;
mod storage;
mod token;

pub use auth_layer::{
    RequirePermission, RequirePermissionLayer, RequirePermissionService, RequireRole,
    RequireRoleLayer, RequireRoleService,
};
pub use context::AxumContext;
pub use extractors::{CurrentLoginId, OptionalLoginId};
pub use layer::{SaTokenLayer, SaTokenService};
pub use request::AxumRequest;
pub use response::AxumResponse;
pub use storage::AxumStorage;
pub use token::{extract_token_from_headers, extract_token_from_request_parts};
