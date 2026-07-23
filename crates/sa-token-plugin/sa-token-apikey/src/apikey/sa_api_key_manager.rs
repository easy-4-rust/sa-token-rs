//! Explicit API Key component manager.

use std::sync::Arc;

use sa_token_core::dao::async_sa_token_dao::AsyncSaTokenDao;

use crate::apikey::config::SaApiKeyConfig;
use crate::apikey::exception::ApiKeyException;
use crate::apikey::loader::SaApiKeyDataLoader;
use crate::apikey::template::{DEFAULT_NAMESPACE, SaApiKeyTemplate};

/// Owns API Key configuration, loader and isolated application service.
pub struct SaApiKeyManager {
    config: Arc<SaApiKeyConfig>,
    loader: Arc<dyn SaApiKeyDataLoader>,
    template: Arc<SaApiKeyTemplate>,
}

impl SaApiKeyManager {
    /// Builds an isolated manager without global mutable state.
    ///
    /// # Errors
    /// Returns an error when the default namespace is invalid.
    pub fn new(
        token_name: impl Into<String>,
        config: Arc<SaApiKeyConfig>,
        dao: Arc<dyn AsyncSaTokenDao>,
        loader: Arc<dyn SaApiKeyDataLoader>,
    ) -> Result<Self, ApiKeyException> {
        let template = Arc::new(SaApiKeyTemplate::new(
            DEFAULT_NAMESPACE,
            token_name,
            Arc::clone(&config),
            dao,
            Arc::clone(&loader),
        )?);
        Ok(Self {
            config,
            loader,
            template,
        })
    }

    /// Returns immutable configuration.
    pub fn config(&self) -> &Arc<SaApiKeyConfig> {
        &self.config
    }

    /// Returns the database loader port.
    pub fn data_loader(&self) -> &Arc<dyn SaApiKeyDataLoader> {
        &self.loader
    }

    /// Returns the isolated API Key service.
    pub fn template(&self) -> &Arc<SaApiKeyTemplate> {
        &self.template
    }
}
