use super::SaOAuth2Exception;

/// Access-token scope validation failure.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("{base}")]
pub struct SaOAuth2AccessTokenScopeException {
    #[source]
    pub base: SaOAuth2Exception,
    pub access_token: Option<String>,
    pub scope: Option<String>,
}

impl SaOAuth2AccessTokenScopeException {
    pub fn new(message: impl Into<String>, code: i32) -> Self {
        Self {
            base: SaOAuth2Exception::new(message, code),
            access_token: None,
            scope: None,
        }
    }

    pub fn with_access_token(mut self, access_token: impl Into<String>) -> Self {
        self.access_token = Some(access_token.into());
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
    /// Returns an access-token scope error.
    pub fn throw_by(flag: bool, message: impl Into<String>, code: i32) -> Result<(), Self> {
        if flag {
            Err(Self::new(message, code))
        } else {
            Ok(())
        }
    }
}
