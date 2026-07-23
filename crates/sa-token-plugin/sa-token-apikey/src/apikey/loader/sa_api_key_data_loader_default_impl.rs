//! Default empty API Key loader.

use async_trait::async_trait;

use super::sa_api_key_data_loader::SaApiKeyDataLoader;
use crate::apikey::exception::ApiKeyException;
use crate::apikey::model::ApiKeyModel;

/// Loader used when applications do not provide an external database.
pub struct SaApiKeyDataLoaderDefaultImpl;

#[async_trait]
impl SaApiKeyDataLoader for SaApiKeyDataLoaderDefaultImpl {
    async fn get_api_key_model_from_database(
        &self,
        _namespace: &str,
        _api_key: &str,
    ) -> Result<Option<ApiKeyModel>, ApiKeyException> {
        Ok(None)
    }
}
