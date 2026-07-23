//! Installs JWT-backed temporary token template (Java `SaTokenPluginForTempForJwt`).

use std::any::Any;
use std::sync::Arc;

use sa_token_core::plugin::sa_token_plugin::SaTokenPlugin;
use sa_token_core::sa_manager::SaManager;

use crate::temp::jwt::SaTempTemplateForJwt;

/// Plugin that replaces the default temp template with JWT implementation.
pub struct SaTokenPluginForTempForJwt;

impl SaTokenPluginForTempForJwt {
    /// Creates the plugin marker.
    pub fn new() -> Self {
        Self
    }
}

impl Default for SaTokenPluginForTempForJwt {
    fn default() -> Self {
        Self::new()
    }
}

impl SaTokenPlugin for SaTokenPluginForTempForJwt {
    fn install(&self) {
        SaManager::set_sa_temp_template(Arc::new(SaTempTemplateForJwt::default()));
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sa_token_core::config::sa_token_config::SaTokenConfig;
    use sa_token_core::temp::sa_temp_util::SaTempUtil;
    use sa_token_dao_memory::SaTokenDaoMemory;
    use serde_json::json;
    use std::sync::Arc;

    #[test]
    fn install_swaps_global_temp_template() {
        SaManager::reset();
        SaManager::set_config(Arc::new(SaTokenConfig {
            jwt_secret_key: "plugin-secret".into(),
            ..Default::default()
        }));
        SaManager::set_sa_token_dao(Arc::new(SaTokenDaoMemory::new()));
        let plugin = SaTokenPluginForTempForJwt::new();
        plugin.install();
        let value = json!("jwt-temp");
        let token = SaTempUtil::create_token(&value, 120).expect("create");
        let parsed = SaTempUtil::parse_token(&token).expect("parse");
        assert_eq!(parsed, Some(value));
    }
}
