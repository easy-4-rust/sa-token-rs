use std::collections::BTreeMap;
use std::sync::Arc;

use serde_json::Value;

use crate::oauth2::consts::{GrantType, SaOAuth2TokenType};
use crate::oauth2::data::model::loader::SaClientModel;
use crate::oauth2::data::model::request::RequestAuthModel;
use crate::oauth2::data::model::{
    AccessTokenModel, ClientTokenModel, CodeModel, RefreshTokenModel,
};

use super::{SaOAuth2DataConverter, SaOAuth2TokenGenerator};

/// Default Java-compatible converter with an isolated token generator.
pub struct SaOAuth2DataConverterDefaultImpl<G> {
    generator: Arc<G>,
}

impl<G> SaOAuth2DataConverterDefaultImpl<G> {
    pub fn new(generator: Arc<G>) -> Self {
        Self { generator }
    }

    fn split_comma(value: &str) -> Vec<String> {
        value
            .split(',')
            .map(str::trim)
            .filter(|item| !item.is_empty())
            .map(str::to_owned)
            .collect()
    }

    fn ttl_to_expire_time_at(ttl: i64, now: i64) -> i64 {
        match ttl {
            -1 => -1,
            value if value < 0 => -2,
            value => value.saturating_mul(1_000).saturating_add(now),
        }
    }

    fn now_millis() -> i64 {
        super::super::model::now_millis()
    }

    fn request_parts(request: &RequestAuthModel) -> (&str, &Value, &[String]) {
        (
            request.client_id.as_deref().unwrap_or_default(),
            request.login_id.as_ref().unwrap_or(&Value::Null),
            request.scopes.as_deref().unwrap_or_default(),
        )
    }
}

impl<G: SaOAuth2TokenGenerator> SaOAuth2DataConverter for SaOAuth2DataConverterDefaultImpl<G> {
    type Error = G::Error;

    fn convert_scope_string_to_list(&self, value: &str) -> Vec<String> {
        Self::split_comma(&value.replace("%20", ",").replace([' ', '+'], ","))
    }

    fn convert_scope_list_to_string(&self, value: &[String]) -> String {
        value.join(",")
    }

    fn convert_redirect_uri_string_to_list(&self, value: &str) -> Vec<String> {
        Self::split_comma(value)
    }

    fn convert_request_auth_to_code(
        &self,
        request: &RequestAuthModel,
    ) -> Result<CodeModel, Self::Error> {
        let (client_id, login_id, scopes) = Self::request_parts(request);
        Ok(CodeModel {
            code: Some(self.generator.create_code(client_id, login_id, scopes)?),
            client_id: request.client_id.clone(),
            scopes: request.scopes.clone(),
            login_id: request.login_id.clone(),
            redirect_uri: request.redirect_uri.clone(),
            nonce: request.nonce.clone(),
            ..Default::default()
        })
    }

    fn convert_request_auth_to_access_token(
        &self,
        request: &RequestAuthModel,
        timeout: i64,
    ) -> Result<AccessTokenModel, Self::Error> {
        let (client_id, login_id, scopes) = Self::request_parts(request);
        Ok(AccessTokenModel {
            access_token: Some(
                self.generator
                    .create_access_token(client_id, login_id, scopes)?,
            ),
            client_id: request.client_id.clone(),
            login_id: request.login_id.clone(),
            scopes: request.scopes.clone(),
            token_type: Some(SaOAuth2TokenType::BEARER_TITLE.into()),
            expires_time: Self::ttl_to_expire_time_at(timeout, Self::now_millis()),
            extra_data: Some(BTreeMap::new()),
            ..Default::default()
        })
    }

    fn convert_code_to_access_token(
        &self,
        code: &CodeModel,
        timeout: i64,
    ) -> Result<AccessTokenModel, Self::Error> {
        let client_id = code.client_id.as_deref().unwrap_or_default();
        let login_id = code.login_id.as_ref().unwrap_or(&Value::Null);
        let scopes = code.scopes.as_deref().unwrap_or_default();
        Ok(AccessTokenModel {
            access_token: Some(
                self.generator
                    .create_access_token(client_id, login_id, scopes)?,
            ),
            client_id: code.client_id.clone(),
            login_id: code.login_id.clone(),
            scopes: code.scopes.clone(),
            token_type: Some(SaOAuth2TokenType::BEARER_TITLE.into()),
            grant_type: Some(GrantType::AUTHORIZATION_CODE.into()),
            expires_time: Self::ttl_to_expire_time_at(timeout, Self::now_millis()),
            extra_data: Some(BTreeMap::new()),
            ..Default::default()
        })
    }

    fn convert_access_token_to_refresh_token(
        &self,
        access_token: &AccessTokenModel,
        timeout: i64,
    ) -> Result<RefreshTokenModel, Self::Error> {
        let client_id = access_token.client_id.as_deref().unwrap_or_default();
        let login_id = access_token.login_id.as_ref().unwrap_or(&Value::Null);
        let scopes = access_token.scopes.as_deref().unwrap_or_default();
        Ok(RefreshTokenModel {
            refresh_token: Some(
                self.generator
                    .create_refresh_token(client_id, login_id, scopes)?,
            ),
            client_id: access_token.client_id.clone(),
            login_id: access_token.login_id.clone(),
            scopes: access_token.scopes.clone(),
            expires_time: Self::ttl_to_expire_time_at(timeout, Self::now_millis()),
            extra_data: access_token.extra_data.clone(),
            ..Default::default()
        })
    }

    fn convert_refresh_token_to_access_token(
        &self,
        refresh_token: &RefreshTokenModel,
        timeout: i64,
    ) -> Result<AccessTokenModel, Self::Error> {
        let client_id = refresh_token.client_id.as_deref().unwrap_or_default();
        let login_id = refresh_token.login_id.as_ref().unwrap_or(&Value::Null);
        let scopes = refresh_token.scopes.as_deref().unwrap_or_default();
        Ok(AccessTokenModel {
            access_token: Some(
                self.generator
                    .create_access_token(client_id, login_id, scopes)?,
            ),
            refresh_token: refresh_token.refresh_token.clone(),
            client_id: refresh_token.client_id.clone(),
            login_id: refresh_token.login_id.clone(),
            scopes: refresh_token.scopes.clone(),
            token_type: Some(SaOAuth2TokenType::BEARER_TITLE.into()),
            grant_type: Some(GrantType::REFRESH_TOKEN.into()),
            extra_data: refresh_token.extra_data.clone(),
            expires_time: Self::ttl_to_expire_time_at(timeout, Self::now_millis()),
            refresh_expires_time: refresh_token.expires_time,
            ..Default::default()
        })
    }

    fn convert_refresh_token_to_refresh_token(
        &self,
        refresh_token: &RefreshTokenModel,
        timeout: i64,
    ) -> Result<RefreshTokenModel, Self::Error> {
        let client_id = refresh_token.client_id.as_deref().unwrap_or_default();
        let login_id = refresh_token.login_id.as_ref().unwrap_or(&Value::Null);
        let scopes = refresh_token.scopes.as_deref().unwrap_or_default();
        Ok(RefreshTokenModel {
            refresh_token: Some(
                self.generator
                    .create_refresh_token(client_id, login_id, scopes)?,
            ),
            expires_time: Self::ttl_to_expire_time_at(timeout, Self::now_millis()),
            client_id: refresh_token.client_id.clone(),
            login_id: refresh_token.login_id.clone(),
            scopes: refresh_token.scopes.clone(),
            extra_data: refresh_token.extra_data.clone(),
            ..Default::default()
        })
    }

    fn convert_sa_client_to_client_token(
        &self,
        client: &SaClientModel,
        scopes: &[String],
    ) -> Result<ClientTokenModel, Self::Error> {
        let client_id = client.client_id.as_deref().unwrap_or_default();
        Ok(ClientTokenModel {
            client_token: Some(self.generator.create_client_token(client_id, scopes)?),
            expires_time: Self::ttl_to_expire_time_at(
                client.client_token_timeout,
                Self::now_millis(),
            ),
            client_id: client.client_id.clone(),
            scopes: Some(scopes.to_vec()),
            token_type: Some(SaOAuth2TokenType::BEARER_TITLE.into()),
            grant_type: Some(GrantType::CLIENT_CREDENTIALS.into()),
            extra_data: Some(BTreeMap::new()),
            ..Default::default()
        })
    }
}
