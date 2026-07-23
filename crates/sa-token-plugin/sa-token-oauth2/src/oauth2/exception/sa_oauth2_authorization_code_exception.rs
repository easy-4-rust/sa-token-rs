use super::SaOAuth2Exception;

/// Authorization-code validation failure.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("{base}")]
pub struct SaOAuth2AuthorizationCodeException {
    #[source]
    pub base: SaOAuth2Exception,
    pub authorization_code: Option<String>,
}

impl SaOAuth2AuthorizationCodeException {
    pub fn new(message: impl Into<String>, code: i32) -> Self {
        Self {
            base: SaOAuth2Exception::new(message, code),
            authorization_code: None,
        }
    }

    pub fn with_authorization_code(mut self, authorization_code: impl Into<String>) -> Self {
        self.authorization_code = Some(authorization_code.into());
        self
    }

    /// Returns an error when `flag` is true.
    ///
    /// # Errors
    ///
    /// Returns an authorization-code error.
    pub fn throw_by(
        flag: bool,
        message: impl Into<String>,
        authorization_code: impl Into<String>,
        code: i32,
    ) -> Result<(), Self> {
        if flag {
            Err(Self::new(message, code).with_authorization_code(authorization_code))
        } else {
            Ok(())
        }
    }
}
