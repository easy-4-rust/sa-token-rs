use super::SaOAuth2Exception;

/// Client-token validation failure.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("{base}")]
pub struct SaOAuth2ClientTokenException {
    #[source]
    pub base: SaOAuth2Exception,
    pub client_token: Option<String>,
}

impl SaOAuth2ClientTokenException {
    pub fn new(message: impl Into<String>, code: i32) -> Self {
        Self {
            base: SaOAuth2Exception::new(message, code),
            client_token: None,
        }
    }

    pub fn with_client_token(mut self, client_token: impl Into<String>) -> Self {
        self.client_token = Some(client_token.into());
        self
    }

    /// Returns an error when `flag` is true.
    ///
    /// # Errors
    ///
    /// Returns a client-token error.
    pub fn throw_by(flag: bool, message: impl Into<String>, code: i32) -> Result<(), Self> {
        if flag {
            Err(Self::new(message, code))
        } else {
            Ok(())
        }
    }
}
