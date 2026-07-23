//! Async API Key application service.

use std::sync::Arc;

use sa_token_core::context::model::sa_request::SaRequest;
use sa_token_core::dao::async_sa_token_dao::AsyncSaTokenDao;
use sa_token_core::exception::SaTokenException;

use crate::apikey::config::SaApiKeyConfig;
use crate::apikey::error::SaApiKeyErrorCode;
use crate::apikey::exception::{ApiKeyException, ApiKeyScopeException};
use crate::apikey::loader::SaApiKeyDataLoader;
use crate::apikey::model::ApiKeyModel;
use crate::apikey::model::api_key_model::now_millis;

/// Default API Key request field and namespace.
pub const DEFAULT_NAMESPACE: &str = "apikey";

/// API Key CRUD, cache and authorization service.
pub struct SaApiKeyTemplate {
    namespace: String,
    token_name: String,
    config: Arc<SaApiKeyConfig>,
    dao: Arc<dyn AsyncSaTokenDao>,
    loader: Arc<dyn SaApiKeyDataLoader>,
}

impl SaApiKeyTemplate {
    /// Creates an isolated API Key service.
    ///
    /// # Errors
    /// Returns an error when the namespace is empty.
    pub fn new(
        namespace: impl Into<String>,
        token_name: impl Into<String>,
        config: Arc<SaApiKeyConfig>,
        dao: Arc<dyn AsyncSaTokenDao>,
        loader: Arc<dyn SaApiKeyDataLoader>,
    ) -> Result<Self, ApiKeyException> {
        let namespace = namespace.into();
        if namespace.is_empty() {
            return Err(ApiKeyException::new(-1, None, "namespace 不能为空"));
        }
        Ok(Self {
            namespace,
            token_name: token_name.into(),
            config,
            dao,
            loader,
        })
    }

    /// Returns the namespace used for request fields and persistence keys.
    pub fn namespace(&self) -> &str {
        &self.namespace
    }

    /// Builds the Java-compatible API Key cache key.
    pub fn splicing_api_key_save_key(&self, api_key: &str) -> String {
        format!("{}:{}:{api_key}", self.token_name, self.namespace)
    }

    /// Loads a key from cache, then database, preserving storage errors.
    pub async fn get_api_key(&self, api_key: &str) -> Result<Option<ApiKeyModel>, ApiKeyException> {
        if api_key.is_empty() {
            return Ok(None);
        }
        let save_key = self.splicing_api_key_save_key(api_key);
        if let Some(value) = self
            .dao
            .get_object(&save_key)
            .await
            .map_err(storage_error)?
        {
            let model = serde_json::from_value(value).map_err(|error| {
                ApiKeyException::new(-1, Some(api_key), format!("API Key 反序列化失败: {error}"))
            })?;
            return Ok(Some(model));
        }
        let model = self
            .loader
            .get_api_key_model_from_database(&self.namespace, api_key)
            .await?;
        if let Some(model) = &model {
            self.save_api_key(model).await?;
        }
        Ok(model)
    }

    /// Validates existence, expiry and enabled state.
    pub async fn check_api_key(&self, api_key: &str) -> Result<ApiKeyModel, ApiKeyException> {
        let model = self.get_api_key(api_key).await?.ok_or_else(|| {
            ApiKeyException::new(SaApiKeyErrorCode::CODE_12301, Some(api_key), "无效 API Key")
        })?;
        if model.time_expired() {
            return Err(ApiKeyException::new(
                SaApiKeyErrorCode::CODE_12302,
                Some(api_key),
                "API Key 已过期",
            ));
        }
        if !model.is_valid {
            return Err(ApiKeyException::new(
                SaApiKeyErrorCode::CODE_12303,
                Some(api_key),
                "API Key 已被禁用",
            ));
        }
        Ok(model)
    }

    /// Persists a model and its account index.
    pub async fn save_api_key(&self, model: &ApiKeyModel) -> Result<(), ApiKeyException> {
        model.validate_for_save()?;
        let save_key = self.splicing_api_key_save_key(&model.api_key);
        if model.time_expired() {
            self.dao
                .delete_object(&save_key)
                .await
                .map_err(storage_error)?;
            return Ok(());
        }
        let value = serde_json::to_value(model)
            .map_err(|error| ApiKeyException::new(-1, Some(&model.api_key), error.to_string()))?;
        self.dao
            .set_object(&save_key, &value, model.expires_in())
            .await
            .map_err(storage_error)?;
        if self.config.is_record_index {
            let index_key = self.index_key(&model.login_id);
            let mut keys = self.read_index(&model.login_id).await?;
            if !keys.contains(&model.api_key) {
                keys.push(model.api_key.clone());
            }
            self.write_index(&index_key, &keys, model.expires_in())
                .await?;
        }
        Ok(())
    }

    /// Deletes one key and removes it from its account index.
    pub async fn delete_api_key(&self, api_key: &str) -> Result<(), ApiKeyException> {
        let Some(model) = self.get_api_key(api_key).await? else {
            return Ok(());
        };
        self.dao
            .delete_object(&self.splicing_api_key_save_key(api_key))
            .await
            .map_err(storage_error)?;
        if self.config.is_record_index {
            let index_key = self.index_key(&model.login_id);
            let mut keys = self.read_index(&model.login_id).await?;
            keys.retain(|key| key != api_key);
            if keys.is_empty() {
                self.dao
                    .delete_object(&index_key)
                    .await
                    .map_err(storage_error)?;
            } else {
                self.write_index(&index_key, &keys, -1).await?;
            }
        }
        Ok(())
    }

    /// Deletes every indexed key for an account.
    pub async fn delete_api_key_by_login_id(&self, login_id: &str) -> Result<(), ApiKeyException> {
        self.require_index()?;
        for api_key in self.read_index(login_id).await? {
            self.dao
                .delete_object(&self.splicing_api_key_save_key(&api_key))
                .await
                .map_err(storage_error)?;
        }
        self.dao
            .delete_object(&self.index_key(login_id))
            .await
            .map_err(storage_error)
    }

    /// Returns all non-expired indexed keys for an account.
    pub async fn get_api_key_list(
        &self,
        login_id: &str,
    ) -> Result<Vec<ApiKeyModel>, ApiKeyException> {
        self.require_index()?;
        let mut models = Vec::new();
        for api_key in self.read_index(login_id).await? {
            if let Some(model) = self.get_api_key(&api_key).await?
                && !model.time_expired()
            {
                models.push(model);
            }
        }
        Ok(models)
    }

    /// Creates a valid model with configured prefix and lifetime.
    pub async fn create_api_key_model(
        &self,
        login_id: impl Into<String>,
    ) -> Result<ApiKeyModel, ApiKeyException> {
        let login_id = login_id.into();
        for _ in 0..100 {
            let api_key = format!(
                "{}{}",
                self.config.prefix,
                sa_token_core::util::sa_fox_util::random_string(36)
            );
            if self.get_api_key(&api_key).await?.is_none() {
                let expires_time = if self.config.timeout == -1 {
                    -1
                } else {
                    now_millis() + self.config.timeout * 1_000
                };
                return Ok(ApiKeyModel {
                    api_key,
                    login_id,
                    expires_time,
                    ..ApiKeyModel::default()
                });
            }
        }
        Err(ApiKeyException::new(
            -1,
            None,
            "生成唯一 API Key 超过最大重试次数",
        ))
    }

    /// Checks that all required scopes are present.
    pub async fn check_api_key_scope(
        &self,
        api_key: &str,
        scopes: &[&str],
    ) -> Result<ApiKeyModel, ApiKeyScopeException> {
        let model = self
            .check_api_key(api_key)
            .await
            .map_err(|error| ApiKeyScopeException {
                code: error.code,
                api_key: api_key.to_string(),
                scope: String::new(),
            })?;
        if let Some(scope) = scopes
            .iter()
            .find(|scope| !model.scopes.iter().any(|owned| owned == **scope))
        {
            return Err(ApiKeyScopeException {
                code: SaApiKeyErrorCode::CODE_12311,
                api_key: api_key.to_string(),
                scope: (*scope).to_string(),
            });
        }
        Ok(model)
    }

    /// Checks that at least one required scope is present.
    pub async fn check_api_key_scope_or(
        &self,
        api_key: &str,
        scopes: &[&str],
    ) -> Result<ApiKeyModel, ApiKeyScopeException> {
        let model = self
            .check_api_key(api_key)
            .await
            .map_err(|error| ApiKeyScopeException {
                code: error.code,
                api_key: api_key.to_string(),
                scope: String::new(),
            })?;
        if scopes.is_empty()
            || scopes
                .iter()
                .any(|scope| model.scopes.iter().any(|owned| owned == *scope))
        {
            return Ok(model);
        }
        Err(ApiKeyScopeException {
            code: SaApiKeyErrorCode::CODE_12311,
            api_key: api_key.to_string(),
            scope: scopes[0].to_string(),
        })
    }

    /// Checks account ownership.
    pub async fn check_api_key_login_id(
        &self,
        api_key: &str,
        login_id: &str,
    ) -> Result<ApiKeyModel, ApiKeyException> {
        let model = self.check_api_key(api_key).await?;
        if model.login_id != login_id {
            return Err(ApiKeyException::new(
                SaApiKeyErrorCode::CODE_12312,
                Some(api_key),
                "API Key 不属于指定用户",
            ));
        }
        Ok(model)
    }

    /// Reads an API Key from parameter, header, then HTTP Basic username.
    pub fn read_api_key_value(&self, request: &dyn SaRequest) -> Option<String> {
        request
            .get_param(&self.namespace)
            .filter(|value| !value.is_empty())
            .or_else(|| {
                request
                    .get_header(&self.namespace)
                    .filter(|value| !value.is_empty())
            })
            .or_else(|| basic_username(request.get_header("Authorization").as_deref()))
    }

    fn require_index(&self) -> Result<(), ApiKeyException> {
        if self.config.is_record_index {
            Ok(())
        } else {
            Err(ApiKeyException::new(
                SaApiKeyErrorCode::CODE_12305,
                None,
                "API Key 索引功能未启用",
            ))
        }
    }

    fn index_key(&self, login_id: &str) -> String {
        format!("{}:{}:index:{login_id}", self.token_name, self.namespace)
    }

    async fn read_index(&self, login_id: &str) -> Result<Vec<String>, ApiKeyException> {
        let value = self
            .dao
            .get_object(&self.index_key(login_id))
            .await
            .map_err(storage_error)?;
        value
            .map(serde_json::from_value)
            .transpose()
            .map(|value| value.unwrap_or_default())
            .map_err(|error| ApiKeyException::new(-1, None, error.to_string()))
    }

    async fn write_index(
        &self,
        index_key: &str,
        keys: &[String],
        timeout: i64,
    ) -> Result<(), ApiKeyException> {
        let value = serde_json::to_value(keys)
            .map_err(|error| ApiKeyException::new(-1, None, error.to_string()))?;
        self.dao
            .set_object(index_key, &value, timeout)
            .await
            .map_err(storage_error)
    }
}

fn storage_error(error: SaTokenException) -> ApiKeyException {
    ApiKeyException::new(-1, None, error.to_string())
}

fn basic_username(authorization: Option<&str>) -> Option<String> {
    use base64::Engine;

    let encoded = authorization?.strip_prefix("Basic ")?;
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(encoded)
        .ok()?;
    let credentials = String::from_utf8(bytes).ok()?;
    let username = credentials.split_once(':')?.0;
    (!username.is_empty()).then(|| username.to_string())
}
