use serde_json::Value;

use crate::oauth2::data::convert::SaOAuth2TokenGenerator;

/// Function port used to create an authorization-code value.
pub trait SaOAuth2CreateCodeValueFunction: SaOAuth2TokenGenerator {
    /// Creates a code value.
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
        self.create_code(client_id, login_id, scopes)
    }
}

impl<T: SaOAuth2TokenGenerator + ?Sized> SaOAuth2CreateCodeValueFunction for T {}
