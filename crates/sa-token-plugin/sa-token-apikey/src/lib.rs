//! API Key plugin migrated from Java `sa-token-apikey`.

pub mod apikey;
pub mod plugin;

pub use apikey::{ApiKeyModel, SaApiKeyConfig, SaApiKeyManager, SaApiKeyTemplate, SaApiKeyUtil};
pub use plugin::SaTokenPluginForApiKey;
