use std::sync::Arc;

use async_trait::async_trait;
use sa_token_core::exception::{SaResult, SaTokenException};

use crate::oauth2::consts::{GrantType, SaOAuth2Param};
use crate::oauth2::data::generate::SaOAuth2DataGenerate;
use crate::oauth2::data::model::AccessTokenModel;
use crate::oauth2::data::model::request::RequestAuthModel;
use crate::oauth2::data::resolver::SaOAuth2Request;

use super::{PasswordAuthResult, SaOAuth2GrantTypeHandlerInterface};

/// Authenticates username/password credentials for the password grant.
pub trait PasswordGrantAuthenticator: Send + Sync {
    /// Authenticates credentials and returns the mapped login ID.
    ///
    /// # Errors
    ///
    /// Returns the application's explicit authentication error.
    fn login(&self, username: &str, password: &str) -> SaResult<PasswordAuthResult>;
}

impl<F> PasswordGrantAuthenticator for F
where
    F: Fn(&str, &str) -> SaResult<PasswordAuthResult> + Send + Sync,
{
    fn login(&self, username: &str, password: &str) -> SaResult<PasswordAuthResult> {
        self(username, password)
    }
}

/// Password grant handler with no insecure default authenticator.
pub struct PasswordGrantTypeHandler {
    generator: Arc<dyn SaOAuth2DataGenerate>,
    authenticator: Arc<dyn PasswordGrantAuthenticator>,
}

impl PasswordGrantTypeHandler {
    pub fn new(
        generator: Arc<dyn SaOAuth2DataGenerate>,
        authenticator: Arc<dyn PasswordGrantAuthenticator>,
    ) -> Self {
        Self {
            generator,
            authenticator,
        }
    }

    fn required_param<'a>(request: &'a SaOAuth2Request, name: &str) -> SaResult<&'a str> {
        request
            .param(name)
            .ok_or_else(|| SaTokenException::with_code(30191, format!("缺少参数: {name}")))
    }
}

#[async_trait]
impl SaOAuth2GrantTypeHandlerInterface for PasswordGrantTypeHandler {
    fn handler_grant_type(&self) -> &str {
        GrantType::PASSWORD
    }

    async fn get_access_token(
        &self,
        request: &SaOAuth2Request,
        client_id: &str,
        scopes: &[String],
    ) -> SaResult<AccessTokenModel> {
        let username = Self::required_param(request, SaOAuth2Param::USERNAME)?;
        let password = Self::required_param(request, SaOAuth2Param::PASSWORD)?;
        let login_id = self
            .authenticator
            .login(username, password)?
            .login_id
            .ok_or_else(|| SaTokenException::with_code(30161, "登录失败"))?;
        let request_auth = RequestAuthModel {
            client_id: Some(client_id.into()),
            login_id: Some(login_id),
            scopes: Some(scopes.to_vec()),
            ..Default::default()
        };
        let mut access = self
            .generator
            .generate_access_token_by_request(&request_auth, true)
            .await?;
        access.grant_type = Some(GrantType::PASSWORD.into());
        Ok(access)
    }
}
