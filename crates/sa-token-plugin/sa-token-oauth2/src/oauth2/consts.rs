//! OAuth2 protocol constants.

pub mod grant_type;
pub mod sa_oauth2_consts;

pub use grant_type::GrantType;
pub use sa_oauth2_consts::{
    SaOAuth2Api, SaOAuth2Consts, SaOAuth2ExtraField, SaOAuth2Param, SaOAuth2ResponseType,
    SaOAuth2TokenType,
};
