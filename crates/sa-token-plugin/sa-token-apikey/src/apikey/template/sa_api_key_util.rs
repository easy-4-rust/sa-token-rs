//! Instance facade corresponding to Java `SaApiKeyUtil`.

use std::sync::Arc;

use sa_token_core::context::model::sa_request::SaRequest;

use crate::apikey::exception::{ApiKeyException, ApiKeyScopeException};
use crate::apikey::model::ApiKeyModel;

use super::sa_api_key_template::SaApiKeyTemplate;

/// High-frequency API Key facade without process-global mutable state.
#[derive(Clone)]
pub struct SaApiKeyUtil {
    template: Arc<SaApiKeyTemplate>,
}

impl SaApiKeyUtil {
    /// Creates a facade for an isolated template.
    pub fn new(template: Arc<SaApiKeyTemplate>) -> Self {
        Self { template }
    }

    /// Reads a key from a request.
    pub fn read_api_key_value(&self, request: &dyn SaRequest) -> Option<String> {
        self.template.read_api_key_value(request)
    }

    /// Validates a key.
    pub async fn check_api_key(&self, api_key: &str) -> Result<ApiKeyModel, ApiKeyException> {
        self.template.check_api_key(api_key).await
    }

    /// Checks all scopes.
    pub async fn check_api_key_scope(
        &self,
        api_key: &str,
        scopes: &[&str],
    ) -> Result<ApiKeyModel, ApiKeyScopeException> {
        self.template.check_api_key_scope(api_key, scopes).await
    }

    /// Checks any scope.
    pub async fn check_api_key_scope_or(
        &self,
        api_key: &str,
        scopes: &[&str],
    ) -> Result<ApiKeyModel, ApiKeyScopeException> {
        self.template.check_api_key_scope_or(api_key, scopes).await
    }
}
