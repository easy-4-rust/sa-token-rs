use std::collections::BTreeMap;
use std::sync::Arc;

use sa_token_core::dao::async_sa_token_dao::AsyncSaTokenDao;
use sa_token_core::exception::{SaResult, SaTokenException};
use sa_token_core::session::sa_session::SaSession;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::oauth2::data::model::{
    AccessTokenModel, ClientTokenModel, CodeModel, RefreshTokenModel,
};

/// Asynchronous OAuth2 persistence operations with Java-compatible keys.
pub struct SaOAuth2Dao {
    dao: Arc<dyn AsyncSaTokenDao>,
    token_name: String,
    code_timeout: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TokenIndexEntry {
    token: String,
    expires_time: i64,
}

#[derive(Clone, Copy)]
enum TokenKind {
    Access,
    Refresh,
    Client,
}

impl SaOAuth2Dao {
    pub const ACCESS_TOKEN_MAP: &'static str = "__HD_ACCESS_TOKEN_MAP";
    pub const REFRESH_TOKEN_MAP: &'static str = "__HD_REFRESH_TOKEN_MAP";
    pub const CLIENT_TOKEN_MAP: &'static str = "__HD_CLIENT_TOKEN_MAP";

    pub fn new(
        dao: Arc<dyn AsyncSaTokenDao>,
        token_name: impl Into<String>,
        code_timeout: i64,
    ) -> Self {
        Self {
            dao,
            token_name: token_name.into(),
            code_timeout,
        }
    }

    fn serialization_error(action: &str, error: impl std::fmt::Display) -> SaTokenException {
        SaTokenException::with_code(-1, format!("OAuth2 {action} failed: {error}"))
    }

    fn now_millis() -> i64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |duration| {
                i64::try_from(duration.as_millis()).unwrap_or(i64::MAX)
            })
    }

    fn ttl_to_expire_time(ttl: i64) -> i64 {
        match ttl {
            -1 => -1,
            value if value < 0 => -2,
            value => Self::now_millis().saturating_add(value.saturating_mul(1_000)),
        }
    }

    fn expire_time_to_ttl(expire_time: i64) -> i64 {
        match expire_time {
            -1 => -1,
            -2 => -2,
            value => {
                let now = Self::now_millis();
                if value < now {
                    -2
                } else {
                    (value - now) / 1_000
                }
            }
        }
    }

    fn raw_session_key(&self, value: &str) -> String {
        format!("{}:raw-session:oauth2:{value}", self.token_name)
    }

    fn read_index(session: &SaSession, key: &str) -> SaResult<Vec<TokenIndexEntry>> {
        session
            .get(key)
            .cloned()
            .map(|value| {
                serde_json::from_value(value)
                    .map_err(|error| Self::serialization_error("index deserialization", error))
            })
            .transpose()
            .map(Option::unwrap_or_default)
    }

    async fn delete_token_by_kind(&self, kind: TokenKind, token: &str) -> SaResult<()> {
        match kind {
            TokenKind::Access => self.delete_access_token(token).await,
            TokenKind::Refresh => self.delete_refresh_token(token).await,
            TokenKind::Client => self.delete_client_token(token).await,
        }
    }

    async fn add_token_index(
        &self,
        raw_value: &str,
        map_key: &str,
        token: &str,
        timeout: i64,
        max_count: i32,
        kind: TokenKind,
    ) -> SaResult<()> {
        let session_id = self.raw_session_key(raw_value);
        let mut session = self.dao.get_session(&session_id).await?.unwrap_or_else(|| {
            let mut value = SaSession::new(&session_id);
            value.set_session_type("oauth2");
            value
        });
        let mut index = Self::read_index(&session, map_key)?;
        index.retain(|entry| Self::expire_time_to_ttl(entry.expires_time) != -2);
        if index.iter().any(|entry| entry.token == token) {
            return Ok(());
        }
        index.push(TokenIndexEntry {
            token: token.to_owned(),
            expires_time: Self::ttl_to_expire_time(timeout),
        });
        if max_count != -1 && index.len() > max_count.max(0) as usize {
            let overflow = index.len() - max_count.max(0) as usize;
            let removed: Vec<_> = index.drain(..overflow).collect();
            for entry in removed {
                self.delete_token_by_kind(kind, &entry.token).await?;
            }
        }
        let max_ttl = index
            .iter()
            .map(|entry| Self::expire_time_to_ttl(entry.expires_time))
            .fold(0, |current, ttl| {
                if current == -1 || ttl == -1 {
                    -1
                } else {
                    current.max(ttl)
                }
            });
        let value = serde_json::to_value(index)
            .map_err(|error| Self::serialization_error("index serialization", error))?;
        session.set(map_key, value);
        self.dao.set_session(&session, max_ttl).await
    }

    async fn delete_token_index_single(
        &self,
        raw_value: &str,
        map_key: &str,
        token: &str,
    ) -> SaResult<()> {
        let session_id = self.raw_session_key(raw_value);
        let Some(mut session) = self.dao.get_session(&session_id).await? else {
            return Ok(());
        };
        let mut index = Self::read_index(&session, map_key)?;
        index.retain(|entry| entry.token != token);
        if index.is_empty() {
            self.dao.delete_session(&session_id).await
        } else {
            let value = serde_json::to_value(index)
                .map_err(|error| Self::serialization_error("index serialization", error))?;
            session.set(map_key, value);
            self.dao.update_session(&session).await
        }
    }

    async fn get_token_index(
        &self,
        raw_value: &str,
        map_key: &str,
    ) -> SaResult<Vec<TokenIndexEntry>> {
        let session_id = self.raw_session_key(raw_value);
        let Some(mut session) = self.dao.get_session(&session_id).await? else {
            return Ok(Vec::new());
        };
        let original = Self::read_index(&session, map_key)?;
        let mut current = original.clone();
        current.retain(|entry| Self::expire_time_to_ttl(entry.expires_time) != -2);
        if current.is_empty() {
            self.dao.delete_session(&session_id).await?;
        } else if current.len() != original.len() {
            let value = serde_json::to_value(&current)
                .map_err(|error| Self::serialization_error("index serialization", error))?;
            session.set(map_key, value);
            self.dao.update_session(&session).await?;
        }
        Ok(current)
    }

    async fn save_model<T: Serialize>(&self, key: String, value: &T, timeout: i64) -> SaResult<()> {
        let json = serde_json::to_value(value)
            .map_err(|error| Self::serialization_error("serialization", error))?;
        self.dao.set_object(&key, &json, timeout).await
    }

    async fn get_model<T: DeserializeOwned>(&self, key: String) -> SaResult<Option<T>> {
        self.dao
            .get_object(&key)
            .await?
            .map(|value| {
                serde_json::from_value(value)
                    .map_err(|error| Self::serialization_error("deserialization", error))
            })
            .transpose()
    }

    pub fn splicing_code_save_key(&self, code: &str) -> String {
        format!("{}:oauth2:code:{code}", self.token_name)
    }

    pub fn splicing_code_index_key(&self, client_id: &str, login_id: &Value) -> String {
        format!(
            "{}:oauth2:code-index:{client_id}:{login_id}",
            self.token_name
        )
    }

    pub fn splicing_access_token_save_key(&self, token: &str) -> String {
        format!("{}:oauth2:access-token:{token}", self.token_name)
    }

    pub fn splicing_access_token_rsd_value(&self, client_id: &str, login_id: &Value) -> String {
        format!("access-token:{client_id}:{login_id}")
    }

    pub fn splicing_refresh_token_save_key(&self, token: &str) -> String {
        format!("{}:oauth2:refresh-token:{token}", self.token_name)
    }

    pub fn splicing_refresh_token_rsd_value(&self, client_id: &str, login_id: &Value) -> String {
        format!("refresh-token:{client_id}:{login_id}")
    }

    pub fn splicing_client_token_save_key(&self, token: &str) -> String {
        format!("{}:oauth2:client-token:{token}", self.token_name)
    }

    pub fn splicing_client_token_rsd_value(&self, client_id: &str) -> String {
        format!("client-token:{client_id}")
    }

    pub fn splicing_grant_scope_key(&self, client_id: &str, login_id: &Value) -> String {
        format!(
            "{}:oauth2:grant-scope:{client_id}:{login_id}",
            self.token_name
        )
    }

    pub fn splicing_state_save_key(&self, state: &str) -> String {
        format!("{}:oauth2:state:{state}", self.token_name)
    }

    pub fn splicing_code_nonce_index_save_key(&self, code: &str) -> String {
        format!("{}:oauth2:code-nonce-index:{code}", self.token_name)
    }

    /// Saves an authorization-code model.
    ///
    /// # Errors
    ///
    /// Returns serialization or backend failures.
    pub async fn save_code(&self, model: &CodeModel) -> SaResult<()> {
        match model.code.as_deref() {
            Some(code) => {
                self.save_model(self.splicing_code_save_key(code), model, self.code_timeout)
                    .await
            }
            None => Ok(()),
        }
    }

    pub async fn get_code(&self, code: &str) -> SaResult<Option<CodeModel>> {
        self.get_model(self.splicing_code_save_key(code)).await
    }

    pub async fn delete_code(&self, code: &str) -> SaResult<()> {
        self.dao
            .delete_object(&self.splicing_code_save_key(code))
            .await
    }

    pub async fn save_code_index(&self, model: &CodeModel) -> SaResult<()> {
        match (
            model.code.as_deref(),
            model.client_id.as_deref(),
            model.login_id.as_ref(),
        ) {
            (Some(code), Some(client_id), Some(login_id)) => {
                self.dao
                    .set(
                        &self.splicing_code_index_key(client_id, login_id),
                        code,
                        self.code_timeout,
                    )
                    .await
            }
            _ => Ok(()),
        }
    }

    pub async fn get_code_value(
        &self,
        client_id: &str,
        login_id: &Value,
    ) -> SaResult<Option<String>> {
        self.dao
            .get(&self.splicing_code_index_key(client_id, login_id))
            .await
    }

    pub async fn delete_code_index(&self, client_id: &str, login_id: &Value) -> SaResult<()> {
        self.dao
            .delete(&self.splicing_code_index_key(client_id, login_id))
            .await
    }

    pub async fn save_access_token(&self, model: &AccessTokenModel) -> SaResult<()> {
        match model.access_token.as_deref() {
            Some(token) => {
                self.save_model(
                    self.splicing_access_token_save_key(token),
                    model,
                    model.expires_in(),
                )
                .await
            }
            None => Ok(()),
        }
    }

    pub async fn get_access_token(&self, token: &str) -> SaResult<Option<AccessTokenModel>> {
        self.get_model(self.splicing_access_token_save_key(token))
            .await
    }

    pub async fn delete_access_token(&self, token: &str) -> SaResult<()> {
        self.dao
            .delete_object(&self.splicing_access_token_save_key(token))
            .await
    }

    pub async fn save_access_token_index_and_adjust(
        &self,
        model: &AccessTokenModel,
        max_count: i32,
    ) -> SaResult<()> {
        match (
            model.access_token.as_deref(),
            model.client_id.as_deref(),
            model.login_id.as_ref(),
        ) {
            (Some(token), Some(client_id), Some(login_id)) => {
                self.add_token_index(
                    &self.splicing_access_token_rsd_value(client_id, login_id),
                    Self::ACCESS_TOKEN_MAP,
                    token,
                    model.expires_in(),
                    max_count,
                    TokenKind::Access,
                )
                .await
            }
            _ => Ok(()),
        }
    }

    pub async fn delete_access_token_index_by_single_data(
        &self,
        client_id: &str,
        login_id: &Value,
        token: &str,
    ) -> SaResult<()> {
        self.delete_token_index_single(
            &self.splicing_access_token_rsd_value(client_id, login_id),
            Self::ACCESS_TOKEN_MAP,
            token,
        )
        .await
    }

    pub async fn delete_access_token_index(
        &self,
        client_id: &str,
        login_id: &Value,
    ) -> SaResult<()> {
        self.dao
            .delete_session(
                &self.raw_session_key(&self.splicing_access_token_rsd_value(client_id, login_id)),
            )
            .await
    }

    pub async fn get_access_token_value_list_from_adjust_after(
        &self,
        client_id: &str,
        login_id: &Value,
    ) -> SaResult<Vec<String>> {
        Ok(self
            .get_token_index(
                &self.splicing_access_token_rsd_value(client_id, login_id),
                Self::ACCESS_TOKEN_MAP,
            )
            .await?
            .into_iter()
            .map(|entry| entry.token)
            .collect())
    }

    pub async fn get_access_token_index_map_from_adjust_after(
        &self,
        client_id: &str,
        login_id: &Value,
    ) -> SaResult<BTreeMap<String, i64>> {
        Ok(self
            .get_token_index(
                &self.splicing_access_token_rsd_value(client_id, login_id),
                Self::ACCESS_TOKEN_MAP,
            )
            .await?
            .into_iter()
            .map(|entry| (entry.token, entry.expires_time))
            .collect())
    }

    pub async fn save_refresh_token(&self, model: &RefreshTokenModel) -> SaResult<()> {
        match model.refresh_token.as_deref() {
            Some(token) => {
                self.save_model(
                    self.splicing_refresh_token_save_key(token),
                    model,
                    model.expires_in(),
                )
                .await
            }
            None => Ok(()),
        }
    }

    pub async fn get_refresh_token(&self, token: &str) -> SaResult<Option<RefreshTokenModel>> {
        self.get_model(self.splicing_refresh_token_save_key(token))
            .await
    }

    pub async fn delete_refresh_token(&self, token: &str) -> SaResult<()> {
        self.dao
            .delete_object(&self.splicing_refresh_token_save_key(token))
            .await
    }

    pub async fn save_refresh_token_index_and_adjust(
        &self,
        model: &RefreshTokenModel,
        max_count: i32,
    ) -> SaResult<()> {
        match (
            model.refresh_token.as_deref(),
            model.client_id.as_deref(),
            model.login_id.as_ref(),
        ) {
            (Some(token), Some(client_id), Some(login_id)) => {
                self.add_token_index(
                    &self.splicing_refresh_token_rsd_value(client_id, login_id),
                    Self::REFRESH_TOKEN_MAP,
                    token,
                    model.expires_in(),
                    max_count,
                    TokenKind::Refresh,
                )
                .await
            }
            _ => Ok(()),
        }
    }

    pub async fn delete_refresh_token_index_by_single_data(
        &self,
        client_id: &str,
        login_id: &Value,
        token: &str,
    ) -> SaResult<()> {
        self.delete_token_index_single(
            &self.splicing_refresh_token_rsd_value(client_id, login_id),
            Self::REFRESH_TOKEN_MAP,
            token,
        )
        .await
    }

    pub async fn delete_refresh_token_index(
        &self,
        client_id: &str,
        login_id: &Value,
    ) -> SaResult<()> {
        self.dao
            .delete_session(
                &self.raw_session_key(&self.splicing_refresh_token_rsd_value(client_id, login_id)),
            )
            .await
    }

    pub async fn get_refresh_token_value_list_from_adjust_after(
        &self,
        client_id: &str,
        login_id: &Value,
    ) -> SaResult<Vec<String>> {
        Ok(self
            .get_token_index(
                &self.splicing_refresh_token_rsd_value(client_id, login_id),
                Self::REFRESH_TOKEN_MAP,
            )
            .await?
            .into_iter()
            .map(|entry| entry.token)
            .collect())
    }

    pub async fn get_refresh_token_index_map_from_adjust_after(
        &self,
        client_id: &str,
        login_id: &Value,
    ) -> SaResult<BTreeMap<String, i64>> {
        Ok(self
            .get_token_index(
                &self.splicing_refresh_token_rsd_value(client_id, login_id),
                Self::REFRESH_TOKEN_MAP,
            )
            .await?
            .into_iter()
            .map(|entry| (entry.token, entry.expires_time))
            .collect())
    }

    pub async fn save_client_token(&self, model: &ClientTokenModel) -> SaResult<()> {
        match model.client_token.as_deref() {
            Some(token) => {
                self.save_model(
                    self.splicing_client_token_save_key(token),
                    model,
                    model.expires_in(),
                )
                .await
            }
            None => Ok(()),
        }
    }

    pub async fn get_client_token(&self, token: &str) -> SaResult<Option<ClientTokenModel>> {
        self.get_model(self.splicing_client_token_save_key(token))
            .await
    }

    pub async fn delete_client_token(&self, token: &str) -> SaResult<()> {
        self.dao
            .delete_object(&self.splicing_client_token_save_key(token))
            .await
    }

    pub async fn save_client_token_index_and_adjust(
        &self,
        model: &ClientTokenModel,
        max_count: i32,
    ) -> SaResult<()> {
        match (model.client_token.as_deref(), model.client_id.as_deref()) {
            (Some(token), Some(client_id)) => {
                self.add_token_index(
                    &self.splicing_client_token_rsd_value(client_id),
                    Self::CLIENT_TOKEN_MAP,
                    token,
                    model.expires_in(),
                    max_count,
                    TokenKind::Client,
                )
                .await
            }
            _ => Ok(()),
        }
    }

    pub async fn delete_client_token_index_by_single_data(
        &self,
        client_id: &str,
        token: &str,
    ) -> SaResult<()> {
        self.delete_token_index_single(
            &self.splicing_client_token_rsd_value(client_id),
            Self::CLIENT_TOKEN_MAP,
            token,
        )
        .await
    }

    pub async fn delete_client_token_index(&self, client_id: &str) -> SaResult<()> {
        self.dao
            .delete_session(&self.raw_session_key(&self.splicing_client_token_rsd_value(client_id)))
            .await
    }

    pub async fn get_client_token_value_list_from_adjust_after(
        &self,
        client_id: &str,
    ) -> SaResult<Vec<String>> {
        Ok(self
            .get_token_index(
                &self.splicing_client_token_rsd_value(client_id),
                Self::CLIENT_TOKEN_MAP,
            )
            .await?
            .into_iter()
            .map(|entry| entry.token)
            .collect())
    }

    pub async fn get_client_token_index_map_from_adjust_after(
        &self,
        client_id: &str,
    ) -> SaResult<BTreeMap<String, i64>> {
        Ok(self
            .get_token_index(
                &self.splicing_client_token_rsd_value(client_id),
                Self::CLIENT_TOKEN_MAP,
            )
            .await?
            .into_iter()
            .map(|entry| (entry.token, entry.expires_time))
            .collect())
    }

    pub async fn save_grant_scope(
        &self,
        client_id: &str,
        login_id: &Value,
        scopes: &[String],
        timeout: i64,
    ) -> SaResult<()> {
        if scopes.is_empty() {
            return Ok(());
        }
        self.dao
            .set(
                &self.splicing_grant_scope_key(client_id, login_id),
                &scopes.join(","),
                timeout,
            )
            .await
    }

    pub async fn get_grant_scope(
        &self,
        client_id: &str,
        login_id: &Value,
    ) -> SaResult<Option<Vec<String>>> {
        Ok(self
            .dao
            .get(&self.splicing_grant_scope_key(client_id, login_id))
            .await?
            .map(|value| {
                value
                    .split(',')
                    .map(str::trim)
                    .filter(|scope| !scope.is_empty())
                    .map(str::to_owned)
                    .collect()
            }))
    }

    pub async fn delete_grant_scope(&self, client_id: &str, login_id: &Value) -> SaResult<()> {
        self.dao
            .delete(&self.splicing_grant_scope_key(client_id, login_id))
            .await
    }

    pub async fn save_state(&self, state: &str) -> SaResult<()> {
        if state.is_empty() {
            return Ok(());
        }
        self.dao
            .set(
                &self.splicing_state_save_key(state),
                state,
                self.code_timeout,
            )
            .await
    }

    pub async fn get_state(&self, state: &str) -> SaResult<Option<String>> {
        self.dao.get(&self.splicing_state_save_key(state)).await
    }

    pub async fn delete_state(&self, state: &str) -> SaResult<()> {
        self.dao.delete(&self.splicing_state_save_key(state)).await
    }

    pub async fn save_code_nonce_index(&self, model: &CodeModel) -> SaResult<()> {
        match (model.code.as_deref(), model.nonce.as_deref()) {
            (Some(code), Some(nonce)) => {
                self.dao
                    .set(
                        &self.splicing_code_nonce_index_save_key(code),
                        nonce,
                        self.code_timeout,
                    )
                    .await
            }
            _ => Ok(()),
        }
    }

    pub async fn get_nonce(&self, code: &str) -> SaResult<Option<String>> {
        self.dao
            .get(&self.splicing_code_nonce_index_save_key(code))
            .await
    }
}
