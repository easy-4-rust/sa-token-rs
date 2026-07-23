use std::sync::Arc;

use sa_token_core::secure::sa_secure_util::SaSecureUtil;

use crate::oauth2::config::SaOAuth2ServerConfig;
use crate::oauth2::data::model::loader::SaClientModel;

use super::SaOAuth2DataLoader;

/// Default loader backed by an isolated server configuration.
pub struct SaOAuth2DataLoaderDefaultImpl {
    config: Arc<SaOAuth2ServerConfig>,
}

impl SaOAuth2DataLoaderDefaultImpl {
    pub fn new(config: Arc<SaOAuth2ServerConfig>) -> Self {
        Self { config }
    }

    fn split_scopes(value: Option<&str>) -> Vec<String> {
        value
            .unwrap_or_default()
            .split(',')
            .map(str::trim)
            .filter(|scope| !scope.is_empty())
            .map(str::to_owned)
            .collect()
    }
}

impl SaOAuth2DataLoader for SaOAuth2DataLoaderDefaultImpl {
    fn get_client_model(&self, client_id: &str) -> Option<SaClientModel> {
        self.config.clients.get(client_id).cloned()
    }

    fn get_openid(&self, client_id: &str, login_id: &str) -> String {
        SaSecureUtil::md5(&format!(
            "{}_{client_id}_{login_id}",
            self.config.openid_digest_prefix
        ))
    }

    fn get_unionid(&self, subject_id: &str, login_id: &str) -> String {
        SaSecureUtil::md5(&format!(
            "{}_{subject_id}_{login_id}",
            self.config.unionid_digest_prefix
        ))
    }

    fn get_higher_scope_list(&self) -> Vec<String> {
        Self::split_scopes(self.config.higher_scope.as_deref())
    }

    fn get_lower_scope_list(&self) -> Vec<String> {
        Self::split_scopes(self.config.lower_scope.as_deref())
    }
}
