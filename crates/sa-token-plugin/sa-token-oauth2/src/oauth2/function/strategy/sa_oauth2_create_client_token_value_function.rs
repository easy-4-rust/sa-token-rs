use crate::oauth2::data::convert::SaOAuth2TokenGenerator;

/// Function port used to create a client-token value.
pub trait SaOAuth2CreateClientTokenValueFunction: SaOAuth2TokenGenerator {
    /// Creates a client-token value.
    ///
    /// # Errors
    ///
    /// Propagates the injected token generator error.
    fn execute(&self, client_id: &str, scopes: &[String]) -> Result<String, Self::Error> {
        self.create_client_token(client_id, scopes)
    }
}

impl<T: SaOAuth2TokenGenerator + ?Sized> SaOAuth2CreateClientTokenValueFunction for T {}
