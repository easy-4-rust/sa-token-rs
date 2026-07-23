//! API Key application services and facade.

pub mod sa_api_key_template;
pub mod sa_api_key_util;

pub use sa_api_key_template::{DEFAULT_NAMESPACE, SaApiKeyTemplate};
pub use sa_api_key_util::SaApiKeyUtil;
