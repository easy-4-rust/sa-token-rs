//! Per-domain golden values for core `SaTokenConfig` + `StpLogic` keys.
//!
//! Fixture is sliced from the master `golden/core.json` by
//! `cargo xtask golden-split`. Keep the keys field list below in sync with
//! `xtask/src/main.rs::domain_keys(DOMAIN_CORE_SA_TOKEN)`.

use sa_token_core::config::sa_token_config::SaTokenConfig;
use sa_token_core::stp::stp_logic::StpLogic;
use serde::Deserialize;

const JAVA_BASELINE: &str = "902886c2149261ccb53a9c982068b7ccd0990237";

#[derive(Deserialize)]
struct CoreSaTokenGolden {
    source_commit: String,
    token_name: String,
    timeout: i64,
    active_timeout: i64,
    is_concurrent: bool,
    max_login_count: i32,
    same_token_timeout: i64,
    token_session_check_login: bool,
    auto_renew: bool,
    token_key: String,
    session_key: String,
    token_session_key: String,
    last_active_key: String,
    switch_key: String,
    disable_key: String,
    disable_service_key: String,
    safe_key: String,
    safe_service_key: String,
}

#[test]
fn core_sa_token_matches_java_baseline() {
    let golden: CoreSaTokenGolden = serde_json::from_str(include_str!("golden/core_sa_token.json"))
        .expect("Java core_sa_token golden must be valid JSON");
    assert_eq!(golden.source_commit, JAVA_BASELINE);

    let config = SaTokenConfig::default();
    assert_eq!(config.token_name, golden.token_name);
    assert_eq!(config.timeout, golden.timeout);
    assert_eq!(config.active_timeout, golden.active_timeout);
    assert_eq!(config.is_concurrent, golden.is_concurrent);
    assert_eq!(config.max_login_count, golden.max_login_count);
    assert_eq!(config.same_token_timeout, golden.same_token_timeout);
    assert_eq!(
        config.token_session_check_login,
        golden.token_session_check_login
    );
    assert_eq!(config.auto_renew, golden.auto_renew);

    let logic = StpLogic::new("login");
    assert_eq!(logic.splicing_key_token_value("TOKEN"), golden.token_key);
    assert_eq!(logic.splicing_key_session("10001"), golden.session_key);
    assert_eq!(
        logic.splicing_key_token_session("TOKEN"),
        golden.token_session_key
    );
    assert_eq!(
        logic.splicing_key_last_active_time("TOKEN"),
        golden.last_active_key
    );
    assert_eq!(logic.splicing_key_switch(), golden.switch_key);
    assert_eq!(
        logic.splicing_key_disable("10001", "login"),
        golden.disable_key
    );
    assert_eq!(
        logic.splicing_key_disable("10001", "account"),
        golden.disable_service_key
    );
    assert_eq!(logic.splicing_key_safe("TOKEN", ""), golden.safe_key);
    assert_eq!(
        logic.splicing_key_safe("TOKEN", "payment"),
        golden.safe_service_key
    );
}
