use super::SaOAuth2Exception;

/// OAuth2 client-model scope validation failure.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("{base}")]
pub struct SaOAuth2ClientModelScopeException {
    #[source]
    pub base: SaOAuth2Exception,
    pub client_id: Option<String>,
    pub scope: Option<String>,
}

impl SaOAuth2ClientModelScopeException {
    pub fn new(message: impl Into<String>, code: i32) -> Self {
        Self {
            base: SaOAuth2Exception::new(message, code),
            client_id: None,
            scope: None,
        }
    }

    pub fn with_client_id(mut self, client_id: impl Into<String>) -> Self {
        self.client_id = Some(client_id.into());
        self
    }

    pub fn with_scope(mut self, scope: impl Into<String>) -> Self {
        self.scope = Some(scope.into());
        self
    }

    /// Returns an error when `flag` is true.
    ///
    /// # Errors
    ///
    /// Returns a client-model scope error.
    pub fn throw_by(flag: bool, message: impl Into<String>, code: i32) -> Result<(), Self> {
        if flag {
            Err(Self::new(message, code))
        } else {
            Ok(())
        }
    }
}
