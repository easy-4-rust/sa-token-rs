//! OAuth2 scope constants and handlers.

pub mod common_scope;
pub mod handler;

pub use common_scope::CommonScope;
pub use handler::{
    OidcScopeContext, OidcScopeContextProvider, OidcScopeHandler, OpenIdScopeHandler,
    SaOAuth2IdTokenGenerator, SaOAuth2ScopeHandlerInterface, UnionIdScopeHandler,
    UserIdScopeHandler,
};
