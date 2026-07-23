use std::sync::Arc;

use async_trait::async_trait;
use sa_token_core::exception::{SaResult, SaTokenException};
use serde_json::Value;
use url::Url;

use crate::oauth2::dao::SaOAuth2Dao;
use crate::oauth2::data::loader::SaOAuth2DataLoader;
use crate::oauth2::data::model::loader::SaClientModel;
use crate::oauth2::data::model::{
    AccessTokenModel, ClientTokenModel, CodeModel, RefreshTokenModel,
};
use crate::oauth2::grant_type::handler::AuthorizationCodeParameterChecker;
use crate::oauth2::strategy::SaOAuth2ClientGrantValidator;

/// Async OAuth2 validation and revocation service.
pub struct SaOAuth2Template {
    loader: Arc<dyn SaOAuth2DataLoader>,
    dao: Arc<SaOAuth2Dao>,
}

impl SaOAuth2Template {
    pub fn new(loader: Arc<dyn SaOAuth2DataLoader>, dao: Arc<SaOAuth2Dao>) -> Self {
        Self { loader, dao }
    }

    fn error(code: i32, message: impl Into<String>) -> SaTokenException {
        SaTokenException::with_code(code, message)
    }

    pub fn get_client_model(&self, client_id: &str) -> Option<SaClientModel> {
        self.loader.get_client_model(client_id)
    }

    pub fn check_client_model(&self, client_id: &str) -> SaResult<SaClientModel> {
        self.get_client_model(client_id)
            .ok_or_else(|| Self::error(30105, format!("无效 client_id: {client_id}")))
    }

    pub fn check_client_secret(
        &self,
        client_id: &str,
        client_secret: &str,
    ) -> SaResult<SaClientModel> {
        let client = self.check_client_model(client_id)?;
        if client.client_secret.as_deref() != Some(client_secret) {
            return Err(Self::error(
                30115,
                format!("无效 client_secret: {client_secret}"),
            ));
        }
        Ok(client)
    }

    pub fn check_contract_scope_for_client(
        &self,
        client: SaClientModel,
        scopes: &[String],
    ) -> SaResult<SaClientModel> {
        for scope in scopes {
            if !client.contract_scopes.iter().any(|item| item == scope) {
                return Err(Self::error(
                    30112,
                    format!("该 client 暂未签约 scope: {scope}"),
                ));
            }
        }
        Ok(client)
    }

    pub fn check_contract_scope(
        &self,
        client_id: &str,
        scopes: &[String],
    ) -> SaResult<SaClientModel> {
        let client = self.check_client_model(client_id)?;
        self.check_contract_scope_for_client(client, scopes)
    }

    pub fn check_client_secret_and_scope(
        &self,
        client_id: &str,
        client_secret: &str,
        scopes: &[String],
    ) -> SaResult<SaClientModel> {
        let client = self.check_client_secret(client_id, client_secret)?;
        self.check_contract_scope_for_client(client, scopes)
    }

    pub fn is_contract_scope(&self, client_id: &str, scopes: &[String]) -> bool {
        self.check_contract_scope(client_id, scopes).is_ok()
    }

    pub fn check_redirect_uri_list_normal(redirect_uris: &[String]) -> SaResult<()> {
        for redirect_uri in redirect_uris {
            if let Some(index) = redirect_uri.find('*')
                && index != redirect_uri.len().saturating_sub(1)
            {
                return Err(Self::error(
                    30114,
                    format!("无效的 allow-url 配置（*通配符只允许出现在最后一位）: {redirect_uri}"),
                ));
            }
        }
        Ok(())
    }

    pub fn check_redirect_uri(&self, client_id: &str, redirect_uri: &str) -> SaResult<()> {
        let normalized = redirect_uri
            .split_once('?')
            .map_or(redirect_uri, |(head, _)| head);
        let parsed = Url::parse(normalized)
            .map_err(|_| Self::error(30113, format!("无效 redirect_url: {redirect_uri}")))?;
        if !matches!(parsed.scheme(), "http" | "https") || parsed.host_str().is_none() {
            return Err(Self::error(
                30113,
                format!("无效 redirect_url: {redirect_uri}"),
            ));
        }
        if normalized.contains('@') {
            return Err(Self::error(
                30113,
                format!("无效 redirect_url（不允许出现@字符）: {normalized}"),
            ));
        }
        let client = self.check_client_model(client_id)?;
        Self::check_redirect_uri_list_normal(&client.allow_redirect_uris)?;
        let allowed = client.allow_redirect_uris.iter().any(|candidate| {
            candidate
                .strip_suffix('*')
                .map_or(candidate == normalized, |prefix| {
                    normalized.starts_with(prefix)
                })
        });
        if !allowed {
            return Err(Self::error(
                30114,
                format!("非法 redirect_url: {normalized}"),
            ));
        }
        Ok(())
    }

    pub async fn is_grant_scope(
        &self,
        login_id: &Value,
        client_id: &str,
        scopes: &[String],
    ) -> SaResult<bool> {
        let granted = self
            .dao
            .get_grant_scope(client_id, login_id)
            .await?
            .unwrap_or_default();
        Ok(scopes.iter().all(|scope| granted.contains(scope)))
    }

    pub async fn save_grant_scope(
        &self,
        client_id: &str,
        login_id: &Value,
        scopes: &[String],
        timeout: i64,
    ) -> SaResult<()> {
        self.dao
            .save_grant_scope(client_id, login_id, scopes, timeout)
            .await
    }

    pub async fn is_need_careful_confirm(
        &self,
        login_id: &Value,
        client_id: &str,
        scopes: &[String],
    ) -> SaResult<bool> {
        if scopes.is_empty() {
            return Ok(false);
        }
        let higher = self.loader.get_higher_scope_list();
        if scopes.iter().any(|scope| higher.contains(scope)) {
            return Ok(true);
        }
        let lower = self.loader.get_lower_scope_list();
        let remaining = scopes
            .iter()
            .filter(|scope| !lower.contains(scope))
            .cloned()
            .collect::<Vec<_>>();
        if remaining.is_empty() {
            return Ok(false);
        }
        Ok(!self.is_grant_scope(login_id, client_id, &remaining).await?)
    }

    pub async fn check_gain_token_param(
        &self,
        code: &str,
        client_id: &str,
        client_secret: &str,
        redirect_uri: Option<&str>,
    ) -> SaResult<CodeModel> {
        let model = self
            .dao
            .get_code(code)
            .await?
            .ok_or_else(|| Self::error(30110, format!("无效 code: {code}")))?;
        if model.client_id.as_deref() != Some(client_id) {
            return Err(Self::error(30105, format!("无效 client_id: {client_id}")));
        }
        self.check_client_secret(client_id, client_secret)?;
        if let Some(redirect_uri) = redirect_uri.filter(|value| !value.is_empty())
            && model.redirect_uri.as_deref() != Some(redirect_uri)
        {
            return Err(Self::error(
                30120,
                format!("无效 redirect_uri: {redirect_uri}"),
            ));
        }
        Ok(model)
    }

    pub async fn check_refresh_token_param(
        &self,
        client_id: &str,
        client_secret: &str,
        refresh_token: &str,
    ) -> SaResult<RefreshTokenModel> {
        let model = self.check_refresh_token(refresh_token).await?;
        if model.client_id.as_deref() != Some(client_id) {
            return Err(Self::error(30122, format!("无效 client_id: {client_id}")));
        }
        self.check_client_secret(client_id, client_secret)?;
        Ok(model)
    }

    pub async fn check_access_token_param(
        &self,
        client_id: &str,
        client_secret: &str,
        access_token: &str,
    ) -> SaResult<AccessTokenModel> {
        let model = self.check_access_token(access_token).await?;
        if model.client_id.as_deref() != Some(client_id) {
            return Err(Self::error(30122, format!("无效 client_id: {client_id}")));
        }
        self.check_client_secret(client_id, client_secret)?;
        Ok(model)
    }

    pub async fn check_code(&self, code: &str) -> SaResult<CodeModel> {
        self.dao
            .get_code(code)
            .await?
            .ok_or_else(|| Self::error(30110, format!("无效 code: {code}")))
    }

    pub async fn check_access_token(&self, token: &str) -> SaResult<AccessTokenModel> {
        self.dao
            .get_access_token(token)
            .await?
            .ok_or_else(|| Self::error(30106, format!("无效 access_token: {token}")))
    }

    pub async fn dao_access_token_exists(&self, token: &str) -> SaResult<bool> {
        Ok(self.dao.get_access_token(token).await?.is_some())
    }

    pub async fn check_refresh_token(&self, token: &str) -> SaResult<RefreshTokenModel> {
        self.dao
            .get_refresh_token(token)
            .await?
            .ok_or_else(|| Self::error(30111, format!("无效 refresh_token: {token}")))
    }

    pub async fn check_client_token(&self, token: &str) -> SaResult<ClientTokenModel> {
        self.dao
            .get_client_token(token)
            .await?
            .ok_or_else(|| Self::error(30107, format!("无效 client_token: {token}")))
    }

    pub async fn check_access_token_scope(
        &self,
        token: &str,
        scopes: &[String],
    ) -> SaResult<AccessTokenModel> {
        let model = self.check_access_token(token).await?;
        for scope in scopes {
            if !model
                .scopes
                .as_ref()
                .is_some_and(|items| items.contains(scope))
            {
                return Err(Self::error(
                    30108,
                    format!("该 access_token 不具备 scope: {scope}"),
                ));
            }
        }
        Ok(model)
    }

    pub async fn check_client_token_scope(
        &self,
        token: &str,
        scopes: &[String],
    ) -> SaResult<ClientTokenModel> {
        let model = self.check_client_token(token).await?;
        for scope in scopes {
            if !model
                .scopes
                .as_ref()
                .is_some_and(|items| items.contains(scope))
            {
                return Err(Self::error(
                    30109,
                    format!("该 client_token 不具备 scope: {scope}"),
                ));
            }
        }
        Ok(model)
    }

    pub async fn revoke_access_token(&self, token: &str) -> SaResult<()> {
        let Some(model) = self.dao.get_access_token(token).await? else {
            return Ok(());
        };
        self.dao.delete_access_token(token).await?;
        if let (Some(client_id), Some(login_id)) =
            (model.client_id.as_deref(), model.login_id.as_ref())
        {
            self.dao
                .delete_access_token_index_by_single_data(client_id, login_id, token)
                .await?;
        }
        Ok(())
    }

    pub async fn revoke_refresh_token(&self, token: &str) -> SaResult<()> {
        let Some(model) = self.dao.get_refresh_token(token).await? else {
            return Ok(());
        };
        self.dao.delete_refresh_token(token).await?;
        if let (Some(client_id), Some(login_id)) =
            (model.client_id.as_deref(), model.login_id.as_ref())
        {
            self.dao
                .delete_refresh_token_index_by_single_data(client_id, login_id, token)
                .await?;
        }
        Ok(())
    }

    pub async fn revoke_client_token(&self, token: &str) -> SaResult<()> {
        let Some(model) = self.dao.get_client_token(token).await? else {
            return Ok(());
        };
        self.dao.delete_client_token(token).await?;
        if let Some(client_id) = model.client_id.as_deref() {
            self.dao
                .delete_client_token_index_by_single_data(client_id, token)
                .await?;
        }
        Ok(())
    }

    pub async fn delete_grant_scope(&self, login_id: &Value, client_id: &str) -> SaResult<()> {
        self.dao.delete_grant_scope(client_id, login_id).await
    }
}

impl SaOAuth2ClientGrantValidator for SaOAuth2Template {
    fn validate(
        &self,
        client_id: &str,
        client_secret: &str,
        scopes: &[String],
    ) -> SaResult<SaClientModel> {
        self.check_client_secret_and_scope(client_id, client_secret, scopes)
    }
}

#[async_trait]
impl AuthorizationCodeParameterChecker for SaOAuth2Template {
    async fn check(
        &self,
        code: &str,
        client_id: &str,
        client_secret: &str,
        redirect_uri: Option<&str>,
    ) -> SaResult<()> {
        self.check_gain_token_param(code, client_id, client_secret, redirect_uri)
            .await
            .map(|_| ())
    }
}
