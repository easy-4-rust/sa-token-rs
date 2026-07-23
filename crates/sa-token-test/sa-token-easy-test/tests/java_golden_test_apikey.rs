//! Per-domain golden values for the API Key template + error codes.
//!
//! Fixture is sliced from the master `golden/core.json` by
//! `cargo xtask golden-split`. Keep keys field list below in sync with
//! `xtask/src/main.rs::domain_keys(DOMAIN_APIKEY)`.

use std::sync::Arc;

use sa_token_apikey::apikey::error::SaApiKeyErrorCode;
use sa_token_apikey::apikey::loader::SaApiKeyDataLoaderDefaultImpl;
use sa_token_apikey::{SaApiKeyConfig, SaApiKeyTemplate};
use sa_token_dao_memory::SaTokenDaoMemory;
use serde::Deserialize;

const JAVA_BASELINE: &str = "902886c2149261ccb53a9c982068b7ccd0990237";

#[derive(Deserialize)]
struct ApikeyGolden {
    source_commit: String,
    api_key_prefix: String,
    api_key_timeout: i64,
    api_key_record_index: bool,
    api_key_save_key: String,
    api_key_invalid_code: i32,
    api_key_scope_code: i32,
}

#[test]
fn apikey_template_and_errors_match_java_baseline() {
    let golden: ApikeyGolden = serde_json::from_str(include_str!("golden/apikey.json"))
        .expect("Java apikey golden must be valid JSON");
    assert_eq!(golden.source_commit, JAVA_BASELINE);

    let api_key_config = SaApiKeyConfig::default();
    assert_eq!(api_key_config.prefix, golden.api_key_prefix);
    assert_eq!(api_key_config.timeout, golden.api_key_timeout);
    assert_eq!(api_key_config.is_record_index, golden.api_key_record_index);

    let api_key_template = SaApiKeyTemplate::new(
        "apikey",
        "satoken",
        Arc::new(api_key_config),
        Arc::new(SaTokenDaoMemory::new()),
        Arc::new(SaApiKeyDataLoaderDefaultImpl),
    )
    .expect("valid API Key template");
    assert_eq!(
        api_key_template.splicing_api_key_save_key("AK-TEST"),
        golden.api_key_save_key
    );
    assert_eq!(SaApiKeyErrorCode::CODE_12301, golden.api_key_invalid_code);
    assert_eq!(SaApiKeyErrorCode::CODE_12311, golden.api_key_scope_code);
}
