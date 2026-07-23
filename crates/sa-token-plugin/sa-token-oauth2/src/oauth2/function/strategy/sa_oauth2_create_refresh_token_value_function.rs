use serde_json::Value;

use crate::oauth2::data::convert::SaOAuth2TokenGenerator;

/// Function port used to create a refresh-token value.
pub trait SaOAuth2CreateRefreshTokenValueFunction: SaOAuth2TokenGenerator {
    /// Creates a refresh-token value.
    ///
    /// # Errors
    ///
    /// Propagates the injected token generator error.
    fn execute(
        &self,
        client_id: &str,
        login_id: &Value,
        scopes: &[String],
    ) -> Result<String, Self::Error> {
        self.create_refresh_token(client_id, login_id, scopes)
    }
}

impl<T: SaOAuth2TokenGenerator + ?Sized> SaOAuth2CreateRefreshTokenValueFunction for T {}
