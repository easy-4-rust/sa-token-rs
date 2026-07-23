//! External API Key source port.

use async_trait::async_trait;

use crate::apikey::exception::ApiKeyException;
use crate::apikey::model::ApiKeyModel;

/// Loads API Key records from an application's database.
#[async_trait]
pub trait SaApiKeyDataLoader: Send + Sync + 'static {
    /// Loads one API Key without applying plugin caching.
    async fn get_api_key_model_from_database(
        &self,
        namespace: &str,
        api_key: &str,
    ) -> Result<Option<ApiKeyModel>, ApiKeyException>;
}
