//! OAuth2 callback ports.

pub mod sa_oauth2_confirm_view_function;
pub mod sa_oauth2_do_login_handle_function;
pub mod sa_oauth2_not_login_view_function;
pub mod strategy;

pub use sa_oauth2_confirm_view_function::SaOAuth2ConfirmViewFunction;
pub use sa_oauth2_do_login_handle_function::SaOAuth2DoLoginHandleFunction;
pub use sa_oauth2_not_login_view_function::SaOAuth2NotLoginViewFunction;
pub use strategy::{
    SaOAuth2CreateAccessTokenValueFunction, SaOAuth2CreateClientTokenValueFunction,
    SaOAuth2CreateCodeValueFunction, SaOAuth2CreateRefreshTokenValueFunction,
    SaOAuth2GrantTypeAuthFunction, SaOAuth2ScopeWorkAccessTokenFunction,
    SaOAuth2ScopeWorkClientTokenFunction,
};
