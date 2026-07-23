use crate::oauth2::data::model::{AccessTokenModel, ClientTokenModel};
use crate::oauth2::exception::SaOAuth2Exception;

/// Contract implemented by every OAuth2 scope handler.
pub trait SaOAuth2ScopeHandlerInterface: Send + Sync {
    fn handler_scope(&self) -> &str;

    /// Enriches an access-token model.
    ///
    /// # Errors
    ///
    /// Returns a protocol error when required client or account data is unavailable.
    fn work_access_token(
        &self,
        access_token: &mut AccessTokenModel,
    ) -> Result<(), SaOAuth2Exception>;

    /// Enriches a client-token model.
    ///
    /// # Errors
    ///
    /// Returns a protocol error when required client data is unavailable.
    fn work_client_token(
        &self,
        client_token: &mut ClientTokenModel,
    ) -> Result<(), SaOAuth2Exception>;

    fn refresh_access_token_is_work(&self) -> bool {
        false
    }
}

pub(crate) fn login_id_string(value: Option<&serde_json::Value>) -> String {
    match value {
        Some(serde_json::Value::String(value)) => value.clone(),
        Some(serde_json::Value::Null) | None => String::new(),
        Some(value) => value.to_string(),
    }
}
