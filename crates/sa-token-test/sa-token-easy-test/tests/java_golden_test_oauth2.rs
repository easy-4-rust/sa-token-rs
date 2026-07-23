//! Per-domain golden values for OAuth2 keys + DAO + openid/unionid loader.
//!
//! Fixture is sliced from the master `golden/core.json` by
//! `cargo xtask golden-split`. Keep keys field list below in sync with
//! `xtask/src/main.rs::domain_keys(DOMAIN_OAUTH2)`.

use std::sync::Arc;

use sa_token_dao_memory::SaTokenDaoMemory;
use sa_token_oauth2::oauth2::config::SaOAuth2OidcConfig;
use sa_token_oauth2::oauth2::consts::{GrantType, SaOAuth2Api, SaOAuth2Consts};
use sa_token_oauth2::oauth2::dao::SaOAuth2Dao;
use sa_token_oauth2::oauth2::data::loader::{SaOAuth2DataLoader, SaOAuth2DataLoaderDefaultImpl};
use sa_token_oauth2::oauth2::error::SaOAuth2ErrorCode;
use serde::Deserialize;

const JAVA_BASELINE: &str = "902886c2149261ccb53a9c982068b7ccd0990237";

#[derive(Deserialize)]
struct Oauth2Golden {
    source_commit: String,
    oauth2_oidc_id_token_timeout: i64,
    oauth2_grant_authorization_code: String,
    oauth2_authorize_api: String,
    oauth2_finally_work_scope: String,
    oauth2_last_error_code: i32,
    oauth2_code_key: String,
    oauth2_code_index_key: String,
    oauth2_access_token_key: String,
    oauth2_access_token_rsd: String,
    oauth2_refresh_token_key: String,
    oauth2_client_token_key: String,
    oauth2_grant_scope_key: String,
    oauth2_state_key: String,
    oauth2_nonce_key: String,
    oauth2_openid: String,
    oauth2_unionid: String,
}

#[test]
fn oauth2_keys_and_loader_match_java_baseline() {
    let golden: Oauth2Golden = serde_json::from_str(include_str!("golden/oauth2.json"))
        .expect("Java oauth2 golden must be valid JSON");
    assert_eq!(golden.source_commit, JAVA_BASELINE);

    assert_eq!(
        SaOAuth2OidcConfig::default().id_token_timeout,
        golden.oauth2_oidc_id_token_timeout
    );
    assert_eq!(
        GrantType::AUTHORIZATION_CODE,
        golden.oauth2_grant_authorization_code
    );
    assert_eq!(SaOAuth2Api::AUTHORIZE, golden.oauth2_authorize_api);
    assert_eq!(
        SaOAuth2Consts::FINALLY_WORK_SCOPE,
        golden.oauth2_finally_work_scope
    );
    assert_eq!(SaOAuth2ErrorCode::CODE_30191, golden.oauth2_last_error_code);

    let oauth2_dao = SaOAuth2Dao::new(Arc::new(SaTokenDaoMemory::new()), "satoken", 300);
    assert_eq!(
        oauth2_dao.splicing_code_save_key("C"),
        golden.oauth2_code_key
    );
    assert_eq!(
        oauth2_dao.splicing_code_index_key("app-a", &serde_json::json!(10001)),
        golden.oauth2_code_index_key
    );
    assert_eq!(
        oauth2_dao.splicing_access_token_save_key("A"),
        golden.oauth2_access_token_key
    );
    assert_eq!(
        oauth2_dao.splicing_access_token_rsd_value("app-a", &serde_json::json!(10001)),
        golden.oauth2_access_token_rsd
    );
    assert_eq!(
        oauth2_dao.splicing_refresh_token_save_key("R"),
        golden.oauth2_refresh_token_key
    );
    assert_eq!(
        oauth2_dao.splicing_client_token_save_key("CT"),
        golden.oauth2_client_token_key
    );
    assert_eq!(
        oauth2_dao.splicing_grant_scope_key("app-a", &serde_json::json!(10001)),
        golden.oauth2_grant_scope_key
    );
    assert_eq!(
        oauth2_dao.splicing_state_save_key("S"),
        golden.oauth2_state_key
    );
    assert_eq!(
        oauth2_dao.splicing_code_nonce_index_save_key("C"),
        golden.oauth2_nonce_key
    );

    let oauth2_loader = SaOAuth2DataLoaderDefaultImpl::new(Arc::new(Default::default()));
    assert_eq!(
        oauth2_loader.get_openid("app-a", "10001"),
        golden.oauth2_openid
    );
    assert_eq!(
        oauth2_loader.get_unionid("subject-a", "10001"),
        golden.oauth2_unionid
    );
}
