use std::sync::Arc;

use sa_token_core::exception::{SaResult, SaTokenException};
use serde_json::Value;

use crate::oauth2::config::SaOAuth2ServerConfig;
use crate::oauth2::consts::{GrantType, SaOAuth2Param, SaOAuth2ResponseType};
use crate::oauth2::data::generate::SaOAuth2DataGenerate;
use crate::oauth2::data::resolver::{SaOAuth2DataResolver, SaOAuth2Request, SaOAuth2Response};
use crate::oauth2::strategy::SaOAuth2Strategy;
use crate::oauth2::template::SaOAuth2Template;

/// Transport-neutral result returned by the OAuth2 processor.
#[derive(Debug, Clone, PartialEq)]
pub enum SaOAuth2ProcessorResponse {
    Data(SaOAuth2Response),
    Redirect(String),
    View(Value),
}

/// OAuth2 protocol controller independent from Axum, Actix Web, or Salvo.
pub struct SaOAuth2ServerProcessor {
    config: Arc<SaOAuth2ServerConfig>,
    resolver: Arc<dyn SaOAuth2DataResolver>,
    generator: Arc<dyn SaOAuth2DataGenerate>,
    template: Arc<SaOAuth2Template>,
    strategy: Arc<SaOAuth2Strategy>,
}

impl SaOAuth2ServerProcessor {
    pub fn new(
        config: Arc<SaOAuth2ServerConfig>,
        resolver: Arc<dyn SaOAuth2DataResolver>,
        generator: Arc<dyn SaOAuth2DataGenerate>,
        template: Arc<SaOAuth2Template>,
        strategy: Arc<SaOAuth2Strategy>,
    ) -> Self {
        Self {
            config,
            resolver,
            generator,
            template,
            strategy,
        }
    }

    fn error(code: i32, message: impl Into<String>) -> SaTokenException {
        SaTokenException::with_code(code, message)
    }

    fn required_param<'a>(request: &'a SaOAuth2Request, name: &str) -> SaResult<&'a str> {
        request
            .param(name)
            .ok_or_else(|| Self::error(30191, format!("缺少参数: {name}")))
    }

    fn parse_scopes(value: Option<&str>) -> Vec<String> {
        value
            .unwrap_or_default()
            .replace("%20", ",")
            .replace([' ', '+'], ",")
            .split(',')
            .map(str::trim)
            .filter(|scope| !scope.is_empty())
            .map(str::to_owned)
            .collect()
    }

    fn check_authorize_response_type(&self, response_type: &str, client_id: &str) -> SaResult<()> {
        let required_grant = match response_type {
            SaOAuth2ResponseType::CODE if self.config.enable_authorization_code => {
                GrantType::AUTHORIZATION_CODE
            }
            SaOAuth2ResponseType::TOKEN if self.config.enable_implicit => GrantType::IMPLICIT,
            SaOAuth2ResponseType::CODE | SaOAuth2ResponseType::TOKEN => {
                return Err(Self::error(30126, "系统未开放的授权模式"));
            }
            _ => {
                return Err(Self::error(
                    30125,
                    format!("无效 response_type: {response_type}"),
                ));
            }
        };
        let client = self.template.check_client_model(client_id)?;
        if !client
            .allow_grant_types
            .iter()
            .any(|grant| grant == required_grant)
        {
            return Err(Self::error(
                30141,
                format!("应用未开放的 grant_type: {required_grant}"),
            ));
        }
        Ok(())
    }

    /// Handles authorization-code and implicit authorization requests.
    ///
    /// # Errors
    ///
    /// Propagates response-type, client, redirect, scope, persistence, and state errors.
    pub async fn authorize(
        &self,
        request: &SaOAuth2Request,
        login_id: Option<Value>,
    ) -> SaResult<SaOAuth2ProcessorResponse> {
        let response_type = Self::required_param(request, SaOAuth2Param::RESPONSE_TYPE)?;
        let client_id = Self::required_param(request, SaOAuth2Param::CLIENT_ID)?;
        self.check_authorize_response_type(response_type, client_id)?;
        let Some(login_id) = login_id else {
            return Ok(SaOAuth2ProcessorResponse::View(
                self.strategy.not_login_view(),
            ));
        };
        let request_auth = self
            .resolver
            .read_request_auth_model(request, login_id.clone());
        let scopes = request_auth.scopes.as_deref().unwrap_or_default();
        let redirect_uri = request_auth.redirect_uri.as_deref().unwrap_or_default();
        self.strategy
            .check_user_authorize_client(&login_id, client_id)?;
        self.template.check_redirect_uri(client_id, redirect_uri)?;
        let client = self.template.check_contract_scope(client_id, scopes)?;
        if self
            .template
            .is_need_careful_confirm(&login_id, client_id, scopes)
            .await?
            && !client.is_auto_confirm
        {
            return Ok(SaOAuth2ProcessorResponse::View(
                self.strategy.confirm_view(client_id, scopes),
            ));
        }
        match response_type {
            SaOAuth2ResponseType::CODE => {
                let code = self.generator.generate_code(&request_auth).await?;
                let code_value = code
                    .code
                    .as_deref()
                    .ok_or_else(|| Self::error(30110, "生成 code 失败"))?;
                let redirect = self
                    .generator
                    .build_redirect_uri(redirect_uri, code_value, request_auth.state.as_deref())
                    .await?;
                Ok(SaOAuth2ProcessorResponse::Redirect(redirect))
            }
            SaOAuth2ResponseType::TOKEN => {
                let access = self
                    .generator
                    .generate_access_token_by_request(&request_auth, false)
                    .await?;
                let token = access
                    .access_token
                    .as_deref()
                    .ok_or_else(|| Self::error(30106, "生成 access_token 失败"))?;
                let redirect = self
                    .generator
                    .build_implicit_redirect_uri(redirect_uri, token, request_auth.state.as_deref())
                    .await?;
                Ok(SaOAuth2ProcessorResponse::Redirect(redirect))
            }
            _ => Err(Self::error(
                30125,
                format!("无效 response_type: {response_type}"),
            )),
        }
    }

    pub async fn token(&self, request: &SaOAuth2Request) -> SaResult<SaOAuth2ProcessorResponse> {
        let access = self.strategy.grant_type_auth(request).await?;
        Ok(SaOAuth2ProcessorResponse::Data(
            self.resolver.build_access_token_return_value(&access),
        ))
    }

    pub async fn refresh(&self, request: &SaOAuth2Request) -> SaResult<SaOAuth2ProcessorResponse> {
        let credentials = self.resolver.read_client_id_and_secret(request)?;
        let client_id = credentials.client_id.as_deref().unwrap_or_default();
        let secret = credentials.client_secret.as_deref().unwrap_or_default();
        let refresh_token = Self::required_param(request, SaOAuth2Param::REFRESH_TOKEN)?;
        self.template
            .check_refresh_token_param(client_id, secret, refresh_token)
            .await?;
        let access = self.generator.refresh_access_token(refresh_token).await?;
        Ok(SaOAuth2ProcessorResponse::Data(
            self.resolver.build_refresh_token_return_value(&access),
        ))
    }

    pub async fn revoke(&self, request: &SaOAuth2Request) -> SaResult<SaOAuth2ProcessorResponse> {
        let access_token = Self::required_param(request, SaOAuth2Param::ACCESS_TOKEN)?;
        if self.template.dao_access_token_exists(access_token).await? {
            let credentials = self.resolver.read_client_id_and_secret(request)?;
            self.template
                .check_access_token_param(
                    credentials.client_id.as_deref().unwrap_or_default(),
                    credentials.client_secret.as_deref().unwrap_or_default(),
                    access_token,
                )
                .await?;
            self.template.revoke_access_token(access_token).await?;
        }
        Ok(SaOAuth2ProcessorResponse::Data(
            self.resolver.build_revoke_token_return_value(),
        ))
    }

    pub fn do_login(&self, request: &SaOAuth2Request) -> SaResult<SaOAuth2ProcessorResponse> {
        let name = Self::required_param(request, SaOAuth2Param::NAME)?;
        let password = Self::required_param(request, SaOAuth2Param::PWD)?;
        Ok(SaOAuth2ProcessorResponse::View(
            self.strategy.do_login(name, password),
        ))
    }

    pub async fn do_confirm(
        &self,
        request: &SaOAuth2Request,
        login_id: Value,
        is_post: bool,
    ) -> SaResult<SaOAuth2ProcessorResponse> {
        if !is_post {
            return Err(Self::error(30151, "无效请求方式"));
        }
        let client_id = Self::required_param(request, SaOAuth2Param::CLIENT_ID)?;
        let scopes = Self::parse_scopes(request.param(SaOAuth2Param::SCOPE));
        self.template
            .save_grant_scope(
                client_id,
                &login_id,
                &scopes,
                self.config.access_token_timeout,
            )
            .await?;
        if request.param(SaOAuth2Param::BUILD_REDIRECT_URI) != Some("true") {
            return Ok(SaOAuth2ProcessorResponse::Data(
                self.resolver.build_revoke_token_return_value(),
            ));
        }
        self.authorize(request, Some(login_id)).await
    }

    pub async fn client_token(
        &self,
        request: &SaOAuth2Request,
    ) -> SaResult<SaOAuth2ProcessorResponse> {
        let grant_type = Self::required_param(request, SaOAuth2Param::GRANT_TYPE)?;
        if grant_type != GrantType::CLIENT_CREDENTIALS {
            return Err(Self::error(30126, format!("无效 grant_type: {grant_type}")));
        }
        if !self.config.enable_client_credentials {
            return Err(Self::error(30126, "系统未开放 client_credentials"));
        }
        let credentials = self.resolver.read_client_id_and_secret(request)?;
        let client_id = credentials.client_id.as_deref().unwrap_or_default();
        let secret = credentials.client_secret.as_deref().unwrap_or_default();
        let scopes = Self::parse_scopes(request.param(SaOAuth2Param::SCOPE));
        let client = self
            .template
            .check_client_secret_and_scope(client_id, secret, &scopes)?;
        if !client
            .allow_grant_types
            .iter()
            .any(|grant| grant == GrantType::CLIENT_CREDENTIALS)
        {
            return Err(Self::error(30141, "应用未开放 client_credentials"));
        }
        let token = self
            .generator
            .generate_client_token(client_id, &scopes)
            .await?;
        Ok(SaOAuth2ProcessorResponse::Data(
            self.resolver.build_client_token_return_value(&token),
        ))
    }
}
