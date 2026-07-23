use super::SaOAuth2Exception;

/// Refresh-token validation failure.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("{base}")]
pub struct SaOAuth2RefreshTokenException {
    #[source]
    pub base: SaOAuth2Exception,
    pub refresh_token: Option<String>,
}

impl SaOAuth2RefreshTokenException {
    pub fn new(message: impl Into<String>, code: i32) -> Self {
        Self {
            base: SaOAuth2Exception::new(message, code),
            refresh_token: None,
        }
    }

    pub fn with_refresh_token(mut self, refresh_token: impl Into<String>) -> Self {
        self.refresh_token = Some(refresh_token.into());
        self
    }

    /// Returns an error when `flag` is true.
    ///
    /// # Errors
    ///
    /// Returns a refresh-token error.
    pub fn throw_by(
        flag: bool,
        message: impl Into<String>,
        refresh_token: Option<String>,
        code: i32,
    ) -> Result<(), Self> {
        if flag {
            let error = Self::new(message, code);
            Err(match refresh_token {
                Some(token) => error.with_refresh_token(token),
                None => error,
            })
        } else {
            Ok(())
        }
    }
}
