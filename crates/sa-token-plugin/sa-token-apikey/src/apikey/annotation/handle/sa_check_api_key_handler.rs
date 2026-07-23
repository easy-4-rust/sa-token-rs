//! Handler for API Key check metadata.

use std::sync::Arc;

use sa_token_core::annotation::sa_mode::SaMode;
use sa_token_core::context::model::sa_request::SaRequest;

use crate::apikey::annotation::SaCheckApiKey;
use crate::apikey::exception::ApiKeyScopeException;
use crate::apikey::template::SaApiKeyUtil;

/// Executes API Key scope checks against an explicit facade.
pub struct SaCheckApiKeyHandler {
    util: Arc<SaApiKeyUtil>,
}

impl SaCheckApiKeyHandler {
    /// Creates a handler.
    pub fn new(util: Arc<SaApiKeyUtil>) -> Self {
        Self { util }
    }

    /// Validates request credentials and configured scopes.
    pub async fn check(
        &self,
        metadata: &SaCheckApiKey,
        request: &dyn SaRequest,
    ) -> Result<(), ApiKeyScopeException> {
        let api_key = self.util.read_api_key_value(request).unwrap_or_default();
        let scopes = metadata
            .scopes
            .iter()
            .map(String::as_str)
            .collect::<Vec<_>>();
        match metadata.mode {
            SaMode::And => self.util.check_api_key_scope(&api_key, &scopes).await?,
            SaMode::Or => self.util.check_api_key_scope_or(&api_key, &scopes).await?,
        };
        Ok(())
    }
}
