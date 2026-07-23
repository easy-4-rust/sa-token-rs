//! OAuth2 endpoint validation markers and handlers.

pub mod handler;
pub mod sa_check_access_token;
pub mod sa_check_client_id_secret;
pub mod sa_check_client_token;

pub use handler::{
    SaCheckAccessTokenHandler, SaCheckClientIdSecretHandler, SaCheckClientTokenHandler,
    SaOAuth2AnnotationValidator,
};
pub use sa_check_access_token::SaCheckAccessToken;
pub use sa_check_client_id_secret::SaCheckClientIdSecret;
pub use sa_check_client_token::SaCheckClientToken;
