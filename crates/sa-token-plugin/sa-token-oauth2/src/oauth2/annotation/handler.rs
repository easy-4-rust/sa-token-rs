//! Runtime-independent annotation validation handlers.

pub mod sa_check_access_token_handler;
pub mod sa_check_client_id_secret_handler;
pub mod sa_check_client_token_handler;

use std::sync::Arc;

pub use sa_check_access_token_handler::SaCheckAccessTokenHandler;
pub use sa_check_client_id_secret_handler::SaCheckClientIdSecretHandler;
pub use sa_check_client_token_handler::SaCheckClientTokenHandler;

/// Validation operations supplied by the active OAuth2 runtime.
pub trait SaOAuth2AnnotationValidator: Send + Sync + 'static {
    type Error;

    fn check_access_token_scope(&self, scope: &[String]) -> Result<(), Self::Error>;
    fn check_client_token_scope(&self, scope: &[String]) -> Result<(), Self::Error>;
    fn check_client_id_secret(&self) -> Result<(), Self::Error>;
}

pub(crate) type Validator<V> = Arc<V>;
