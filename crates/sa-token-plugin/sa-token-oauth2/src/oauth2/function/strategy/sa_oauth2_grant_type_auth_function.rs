use crate::oauth2::data::model::AccessTokenModel;
use crate::oauth2::data::resolver::SaOAuth2Request;

/// Framework-neutral grant-type authentication function.
pub trait SaOAuth2GrantTypeAuthFunction: Send + Sync {
    type Error;

    /// Authenticates the request and returns its access-token model.
    ///
    /// # Errors
    ///
    /// Returns the protocol error produced by the injected function.
    fn execute(&self, request: &SaOAuth2Request) -> Result<AccessTokenModel, Self::Error>;
}

impl<F, E> SaOAuth2GrantTypeAuthFunction for F
where
    F: Fn(&SaOAuth2Request) -> Result<AccessTokenModel, E> + Send + Sync,
{
    type Error = E;

    fn execute(&self, request: &SaOAuth2Request) -> Result<AccessTokenModel, Self::Error> {
        self(request)
    }
}
