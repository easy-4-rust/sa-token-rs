use std::fmt::Display;
use std::sync::Arc;

use async_trait::async_trait;
use sa_token_core::exception::{SaResult, SaTokenException};
use serde_json::Value;

use crate::oauth2::dao::SaOAuth2Dao;
use crate::oauth2::data::convert::SaOAuth2DataConverter;
use crate::oauth2::data::loader::SaOAuth2DataLoader;
use crate::oauth2::data::model::request::RequestAuthModel;
use crate::oauth2::data::model::{AccessTokenModel, ClientTokenModel, CodeModel};

use super::{SaOAuth2DataGenerate, SaOAuth2GenerateHooks};

/// Default generator composed from isolated DAO, converter, loader, and hooks.
pub struct SaOAuth2DataGenerateDefaultImpl<C, L, H> {
    dao: Arc<SaOAuth2Dao>,
    converter: Arc<C>,
    loader: Arc<L>,
    hooks: Arc<H>,
}

impl<C, L, H> SaOAuth2DataGenerateDefaultImpl<C, L, H> {
    pub fn new(dao: Arc<SaOAuth2Dao>, converter: Arc<C>, loader: Arc<L>, hooks: Arc<H>) -> Self {
        Self {
            dao,
            converter,
            loader,
            hooks,
        }
    }

    fn protocol_error(code: i32, message: impl Into<String>) -> SaTokenException {
        SaTokenException::with_code(code, message)
    }

    fn join_parameter(url: &str, delimiter: char, key: &str, value: &str) -> String {
        let separator = if url.contains(delimiter) {
            if url.ends_with(delimiter) || url.ends_with('&') {
                ""
            } else {
                "&"
            }
        } else if delimiter == '?' {
            "?"
        } else {
            "#"
        };
        format!("{url}{separator}{key}={value}")
    }

    fn required_parts(request: &RequestAuthModel) -> SaResult<(&str, &Value)> {
        let client_id = request
            .client_id
            .as_deref()
            .ok_or_else(|| Self::protocol_error(30101, "client_id 不可为空"))?;
        let login_id = request
            .login_id
            .as_ref()
            .ok_or_else(|| Self::protocol_error(30104, "LoginId 不可为空"))?;
        Ok((client_id, login_id))
    }
}

#[async_trait]
impl<C, L, H> SaOAuth2DataGenerate for SaOAuth2DataGenerateDefaultImpl<C, L, H>
where
    C: SaOAuth2DataConverter + Send + Sync,
    C::Error: Display + Send + Sync,
    L: SaOAuth2DataLoader + Send + Sync,
    H: SaOAuth2GenerateHooks + Send + Sync,
{
    async fn generate_code(&self, request: &RequestAuthModel) -> SaResult<CodeModel> {
        request
            .check_model()
            .map_err(|error| Self::protocol_error(error.code, error.message))?;
        let (client_id, login_id) = Self::required_parts(request)?;
        if let Some(old_code) = self.dao.get_code_value(client_id, login_id).await? {
            self.dao.delete_code(&old_code).await?;
        }
        let model = self
            .converter
            .convert_request_auth_to_code(request)
            .map_err(|error| Self::protocol_error(-1, error.to_string()))?;
        self.dao.save_code(&model).await?;
        self.dao.save_code_index(&model).await?;
        self.dao.save_code_nonce_index(&model).await?;
        Ok(model)
    }

    async fn generate_access_token(&self, code: &str) -> SaResult<AccessTokenModel> {
        let code_model = self
            .dao
            .get_code(code)
            .await?
            .ok_or_else(|| Self::protocol_error(30110, format!("无效 code: {code}")))?;
        let client_id = code_model.client_id.as_deref().unwrap_or_default();
        let login_id = code_model.login_id.as_ref().unwrap_or(&Value::Null);
        self.hooks
            .user_authorize_client_check(login_id, client_id)?;
        let client = self
            .loader
            .get_client_model_not_null(client_id)
            .map_err(|error| Self::protocol_error(error.base.code, error.base.message))?;
        let mut access = self
            .converter
            .convert_code_to_access_token(&code_model, client.access_token_timeout)
            .map_err(|error| Self::protocol_error(-1, error.to_string()))?;
        self.hooks.work_access_token_by_scope(&mut access)?;
        let refresh = self
            .converter
            .convert_access_token_to_refresh_token(&access, client.refresh_token_timeout)
            .map_err(|error| Self::protocol_error(-1, error.to_string()))?;
        access.refresh_token.clone_from(&refresh.refresh_token);
        access.refresh_expires_time = refresh.expires_time;
        self.dao.save_access_token(&access).await?;
        self.dao
            .save_access_token_index_and_adjust(&access, client.max_access_token_count)
            .await?;
        self.dao.save_refresh_token(&refresh).await?;
        self.dao
            .save_refresh_token_index_and_adjust(&refresh, client.max_refresh_token_count)
            .await?;
        self.dao.delete_code(code).await?;
        self.dao.delete_code_index(client_id, login_id).await?;
        Ok(access)
    }

    async fn refresh_access_token(&self, refresh_token: &str) -> SaResult<AccessTokenModel> {
        let mut refresh = self
            .dao
            .get_refresh_token(refresh_token)
            .await?
            .ok_or_else(|| {
                Self::protocol_error(30111, format!("无效 refresh_token: {refresh_token}"))
            })?;
        let client_id = refresh.client_id.as_deref().unwrap_or_default();
        let login_id = refresh.login_id.as_ref().unwrap_or(&Value::Null);
        self.hooks
            .user_authorize_client_check(login_id, client_id)?;
        let client = self
            .loader
            .get_client_model_not_null(client_id)
            .map_err(|error| Self::protocol_error(error.base.code, error.base.message))?;
        if client.is_new_refresh {
            refresh = self
                .converter
                .convert_refresh_token_to_refresh_token(&refresh, client.refresh_token_timeout)
                .map_err(|error| Self::protocol_error(-1, error.to_string()))?;
            self.dao.save_refresh_token(&refresh).await?;
            self.dao
                .save_refresh_token_index_and_adjust(&refresh, client.max_refresh_token_count)
                .await?;
        }
        let mut access = self
            .converter
            .convert_refresh_token_to_access_token(&refresh, client.access_token_timeout)
            .map_err(|error| Self::protocol_error(-1, error.to_string()))?;
        self.hooks.refresh_access_token_work_by_scope(&mut access)?;
        self.dao.save_access_token(&access).await?;
        self.dao
            .save_access_token_index_and_adjust(&access, client.max_access_token_count)
            .await?;
        Ok(access)
    }

    async fn generate_access_token_by_request(
        &self,
        request: &RequestAuthModel,
        create_refresh_token: bool,
    ) -> SaResult<AccessTokenModel> {
        let (client_id, login_id) = Self::required_parts(request)?;
        if request.scopes.as_ref().is_none_or(Vec::is_empty) {
            return Err(Self::protocol_error(30102, "Scope 不可为空"));
        }
        self.hooks
            .user_authorize_client_check(login_id, client_id)?;
        let client = self
            .loader
            .get_client_model_not_null(client_id)
            .map_err(|error| Self::protocol_error(error.base.code, error.base.message))?;
        let mut access = self
            .converter
            .convert_request_auth_to_access_token(request, client.access_token_timeout)
            .map_err(|error| Self::protocol_error(-1, error.to_string()))?;
        self.hooks.work_access_token_by_scope(&mut access)?;
        if create_refresh_token {
            let refresh = self
                .converter
                .convert_access_token_to_refresh_token(&access, client.refresh_token_timeout)
                .map_err(|error| Self::protocol_error(-1, error.to_string()))?;
            access.refresh_token.clone_from(&refresh.refresh_token);
            access.refresh_expires_time = refresh.expires_time;
            self.dao.save_refresh_token(&refresh).await?;
            self.dao
                .save_refresh_token_index_and_adjust(&refresh, client.max_refresh_token_count)
                .await?;
        }
        self.dao.save_access_token(&access).await?;
        self.dao
            .save_access_token_index_and_adjust(&access, client.max_access_token_count)
            .await?;
        Ok(access)
    }

    async fn generate_client_token(
        &self,
        client_id: &str,
        scopes: &[String],
    ) -> SaResult<ClientTokenModel> {
        let client = self
            .loader
            .get_client_model_not_null(client_id)
            .map_err(|error| Self::protocol_error(error.base.code, error.base.message))?;
        let mut token = self
            .converter
            .convert_sa_client_to_client_token(&client, scopes)
            .map_err(|error| Self::protocol_error(-1, error.to_string()))?;
        self.hooks.work_client_token_by_scope(&mut token)?;
        self.dao.save_client_token(&token).await?;
        self.dao
            .save_client_token_index_and_adjust(&token, client.max_client_token_count)
            .await?;
        Ok(token)
    }

    async fn build_redirect_uri(
        &self,
        redirect_uri: &str,
        code: &str,
        state: Option<&str>,
    ) -> SaResult<String> {
        let mut url = Self::join_parameter(redirect_uri, '?', "code", code);
        if let Some(state) = state.filter(|value| !value.is_empty()) {
            self.check_state(state).await?;
            url = Self::join_parameter(&url, '?', "state", state);
        }
        Ok(url)
    }

    async fn build_implicit_redirect_uri(
        &self,
        redirect_uri: &str,
        token: &str,
        state: Option<&str>,
    ) -> SaResult<String> {
        let mut url = Self::join_parameter(redirect_uri, '#', "token", token);
        if let Some(state) = state.filter(|value| !value.is_empty()) {
            self.check_state(state).await?;
            url = Self::join_parameter(&url, '#', "state", state);
        }
        Ok(url)
    }

    async fn check_state(&self, state: &str) -> SaResult<()> {
        if self.dao.get_state(state).await?.is_some() {
            return Err(Self::protocol_error(
                30127,
                format!("多次请求的 state 不可重复: {state}"),
            ));
        }
        self.dao.save_state(state).await
    }
}
