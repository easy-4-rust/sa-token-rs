//! 对应 Java：`sa-token-temp-jwt-test` / `SaTempTemplateForJwtTest.java`

use std::sync::{Arc, Mutex, MutexGuard};

use sa_token::prelude::*;
use sa_token_core::plugin::sa_token_plugin::SaTokenPlugin;
use sa_token_core::temp::SaTempTemplate;
use sa_token_core::temp::sa_temp_util::SaTempUtil;
use sa_token_temp_jwt::{
    SaTempJwtErrorCode, SaTempTemplateForJwt, SaTokenPluginForTempForJwt,
};
use serde_json::json;

static TEST_LOCK: Mutex<()> = Mutex::new(());

/// 初始化测试环境并安装 temp-jwt 插件。
fn setup() -> MutexGuard<'static, ()> {
    let guard = TEST_LOCK
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    SaManager::reset();
    SaManager::set_config(Arc::new(SaTokenConfig {
        jwt_secret_key: "temp-jwt-secret".into(),
        ..Default::default()
    }));
    SaManager::set_sa_token_dao(Arc::new(SaTokenDaoMemory::new()));
    SaTokenContextMockUtil::set_mock_context();
    SaManager::put_stp_logic(Arc::new(StpLogic::new("login")));
    SaTokenPluginForTempForJwt::new().install();
    guard
}

/// 对应 `SaTempTemplateForJwtTest`：临时 Token 创建与解析。
#[test]
fn sa_temp_create_and_parse() {
    let _guard = setup();
    let value = json!("payload-10001");
    let token = SaTempUtil::create_token(&value, 60).expect("create temp token");
    assert!(!token.is_empty());
    let parsed = SaTempUtil::parse_token(&token).expect("parse");
    assert_eq!(parsed, Some(value));
}

/// JWT temp 模板禁用 delete/list 能力。
#[test]
fn jwt_temp_template_disables_delete_and_list() {
    let _guard = setup();
    let tpl = SaTempTemplateForJwt::default();
    let err = tpl.delete_token("x").expect_err("delete disabled");
    assert_eq!(err.code(), SaTempJwtErrorCode::CODE_30302);
    let list_err = tpl
        .get_temp_token_list(&json!("x"))
        .expect_err("list disabled");
    assert_eq!(list_err.code(), SaTempJwtErrorCode::CODE_30304);
}

/// 未配置 jwtSecretKey 时使用 30301。
#[test]
fn missing_jwt_secret_key_returns_30301() {
    let _guard = setup();
    SaManager::set_config(Arc::new(SaTokenConfig::default()));
    let err = SaTempUtil::create_token(&json!("x"), 60).expect_err("missing secret");
    assert_eq!(err.code(), SaTempJwtErrorCode::CODE_30301);
}
