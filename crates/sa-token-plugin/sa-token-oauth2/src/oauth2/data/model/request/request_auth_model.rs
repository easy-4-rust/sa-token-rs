use std::fmt;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::oauth2::exception::SaOAuth2Exception;

/// Parameters collected during an OAuth2 authorization request.
#[derive(Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct RequestAuthModel {
    pub client_id: Option<String>,
    pub scopes: Option<Vec<String>>,
    pub login_id: Option<Value>,
    pub redirect_uri: Option<String>,
    pub response_type: Option<String>,
    pub state: Option<String>,
    pub nonce: Option<String>,
}

impl RequestAuthModel {
    /// Validates the four fields required by the Java authorization pipeline.
    ///
    /// # Errors
    ///
    /// Returns codes 30101 through 30104 for the first missing required field.
    pub fn check_model(&self) -> Result<(), SaOAuth2Exception> {
        if self.client_id.as_deref().is_none_or(str::is_empty) {
            return Err(SaOAuth2Exception::new("client_id 不可为空", 30101));
        }
        if self.scopes.as_ref().is_none_or(Vec::is_empty) {
            return Err(SaOAuth2Exception::new("scope 不可为空", 30102));
        }
        if self.redirect_uri.as_deref().is_none_or(str::is_empty) {
            return Err(SaOAuth2Exception::new("redirect_uri 不可为空", 30103));
        }
        if self.login_id.is_none() {
            return Err(SaOAuth2Exception::new("LoginId 不可为空", 30104));
        }
        Ok(())
    }
}

impl fmt::Debug for RequestAuthModel {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("RequestAuthModel")
            .field("client_id", &self.client_id)
            .field("scopes", &self.scopes)
            .field("login_id", &self.login_id)
            .field("redirect_uri", &self.redirect_uri)
            .field("response_type", &self.response_type)
            .field("state", &self.state.as_ref().map(|_| "***"))
            .field("nonce", &self.nonce.as_ref().map(|_| "***"))
            .finish()
    }
}
