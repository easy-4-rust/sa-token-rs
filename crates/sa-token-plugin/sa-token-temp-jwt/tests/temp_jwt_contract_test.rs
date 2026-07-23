//! Temp-jwt plugin contract tests.

use sa_token_core::config::sa_token_config::SaTokenConfig;
use sa_token_core::plugin::sa_token_plugin::SaTokenPlugin;
use sa_token_core::sa_manager::SaManager;
use sa_token_core::temp::sa_temp_util::SaTempUtil;
use sa_token_dao_memory::SaTokenDaoMemory;
use sa_token_temp_jwt::{SaTempJwtErrorCode, SaTokenPluginForTempForJwt};
use serde_json::json;
use std::sync::Arc;

#[test]
fn plugin_install_enables_jwt_temp_tokens() {
    SaManager::reset();
    SaManager::set_config(Arc::new(SaTokenConfig {
        jwt_secret_key: "contract-secret".into(),
        ..Default::default()
    }));
    SaManager::set_sa_token_dao(Arc::new(SaTokenDaoMemory::new()));
    SaTokenPluginForTempForJwt::new().install();
    let value = json!("contract");
    let token = SaTempUtil::create_token(&value, 30).expect("create");
    let parsed = SaTempUtil::parse_token(&token).expect("parse");
    assert_eq!(parsed, Some(value));
}

#[test]
fn delete_after_install_uses_disabled_code() {
    SaManager::reset();
    SaManager::set_config(Arc::new(SaTokenConfig {
        jwt_secret_key: "contract-secret".into(),
        ..Default::default()
    }));
    SaManager::set_sa_token_dao(Arc::new(SaTokenDaoMemory::new()));
    SaTokenPluginForTempForJwt::new().install();
    let token = SaTempUtil::create_token(&json!("x"), 30).expect("create");
    let err = SaTempUtil::delete_token(&token).expect_err("delete disabled");
    assert_eq!(err.code(), SaTempJwtErrorCode::CODE_30302);
}
