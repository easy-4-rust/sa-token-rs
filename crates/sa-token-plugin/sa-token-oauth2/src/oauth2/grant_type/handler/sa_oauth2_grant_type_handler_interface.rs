use async_trait::async_trait;
use sa_token_core::exception::SaResult;

use crate::oauth2::data::model::AccessTokenModel;
use crate::oauth2::data::resolver::SaOAuth2Request;

/// Object-safe asynchronous grant-type handler contract.
#[async_trait]
pub trait SaOAuth2GrantTypeHandlerInterface: Send + Sync {
    fn handler_grant_type(&self) -> &str;

    /// Creates an access token for the request's grant.
    ///
    /// # Errors
    ///
    /// Returns parameter, authentication, persistence, or protocol errors unchanged.
    async fn get_access_token(
        &self,
        request: &SaOAuth2Request,
        client_id: &str,
        scopes: &[String],
    ) -> SaResult<AccessTokenModel>;
}
