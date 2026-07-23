//! Built-in scope handlers.

pub mod oidc_scope_handler;
pub mod open_id_scope_handler;
pub mod sa_oauth2_scope_handler_interface;
pub mod union_id_scope_handler;
pub mod user_id_scope_handler;

pub use oidc_scope_handler::{
    OidcScopeContext, OidcScopeContextProvider, OidcScopeHandler, SaOAuth2IdTokenGenerator,
};
pub use open_id_scope_handler::OpenIdScopeHandler;
pub use sa_oauth2_scope_handler_interface::SaOAuth2ScopeHandlerInterface;
pub use union_id_scope_handler::UnionIdScopeHandler;
pub use user_id_scope_handler::UserIdScopeHandler;
