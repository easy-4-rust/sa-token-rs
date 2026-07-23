use std::sync::Arc;

use async_trait::async_trait;
use sa_token_core::exception::{SaResult, SaTokenException};

use crate::oauth2::consts::{GrantType, SaOAuth2Param};
use crate::oauth2::data::generate::SaOAuth2DataGenerate;
use crate::oauth2::data::model::AccessTokenModel;
use crate::oauth2::data::resolver::{SaOAuth2DataResolver, SaOAuth2Request};

use super::SaOAuth2GrantTypeHandlerInterface;

/// Validates the authorization code, client credentials, and redirect URI.
#[async_trait]
pub trait AuthorizationCodeParameterChecker: Send + Sync {
    /// Checks the token-exchange parameters.
    ///
    /// # Errors
    ///
    /// Returns a Java-compatible OAuth2 protocol error.
    async fn check(
        &self,
        code: &str,
        client_id: &str,
        client_secret: &str,
        redirect_uri: Option<&str>,
    ) -> SaResult<()>;
}

/// Authorization-code grant handler with explicit resolver and validator ports.
pub struct AuthorizationCodeGrantTypeHandler {
    resolver: Arc<dyn SaOAuth2DataResolver>,
    generator: Arc<dyn SaOAuth2DataGenerate>,
    checker: Arc<dyn AuthorizationCodeParameterChecker>,
}

impl AuthorizationCodeGrantTypeHandler {
    pub fn new(
        resolver: Arc<dyn SaOAuth2DataResolver>,
        generator: Arc<dyn SaOAuth2DataGenerate>,
        checker: Arc<dyn AuthorizationCodeParameterChecker>,
    ) -> Self {
        Self {
            resolver,
            generator,
            checker,
        }
    }

    fn required_param<'a>(request: &'a SaOAuth2Request, name: &str) -> SaResult<&'a str> {
        request
            .param(name)
            .ok_or_else(|| SaTokenException::with_code(30191, format!("缺少参数: {name}")))
    }
}

#[async_trait]
impl SaOAuth2GrantTypeHandlerInterface for AuthorizationCodeGrantTypeHandler {
    fn handler_grant_type(&self) -> &str {
        GrantType::AUTHORIZATION_CODE
    }

    async fn get_access_token(
        &self,
        request: &SaOAuth2Request,
        client_id: &str,
        _: &[String],
    ) -> SaResult<AccessTokenModel> {
        let credentials = self.resolver.read_client_id_and_secret(request)?;
        let client_secret = credentials.client_secret.as_deref().unwrap_or_default();
        let code = Self::required_param(request, SaOAuth2Param::CODE)?;
        self.checker
            .check(
                code,
                client_id,
                client_secret,
                request.param(SaOAuth2Param::REDIRECT_URI),
            )
            .await?;
        self.generator.generate_access_token(code).await
    }
}
