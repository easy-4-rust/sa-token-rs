/// Base OAuth2 protocol error carrying the Java-compatible detail code.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("{message} (code {code})")]
pub struct SaOAuth2Exception {
    pub message: String,
    pub code: i32,
}

impl SaOAuth2Exception {
    pub fn new(message: impl Into<String>, code: i32) -> Self {
        Self {
            message: message.into(),
            code,
        }
    }

    /// Returns an error when `flag` is true.
    ///
    /// # Errors
    ///
    /// Returns an OAuth2 error containing `message` and `code`.
    pub fn throw_by(flag: bool, message: impl Into<String>, code: i32) -> Result<(), Self> {
        if flag {
            Err(Self::new(message, code))
        } else {
            Ok(())
        }
    }
}
