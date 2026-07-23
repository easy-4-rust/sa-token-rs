//! Per-domain golden values for SSO client/server templates + helpers.
//!
//! Fixture is sliced from the master `golden/core.json` by
//! `cargo xtask golden-split`. Keep keys field list below in sync with
//! `xtask/src/main.rs::domain_keys(DOMAIN_SSO)`.

use std::sync::Arc;

use sa_token_core::dao::sa_token_dao::SaTokenDao;
use sa_token_dao_memory::SaTokenDaoMemory;
use sa_token_sign::SaSignConfig;
use sa_token_sso::sso::config::{SaSsoClientConfig, SaSsoServerConfig};
use sa_token_sso::sso::error::SaSsoErrorCode;
use sa_token_sso::sso::name::{ApiName, ParamName};
use sa_token_sso::sso::strategy::{SaSsoClientStrategy, SaSsoServerStrategy};
use sa_token_sso::sso::template::{SaSsoClientTemplate, SaSsoServerAuth, SaSsoServerTemplate};
use sa_token_sso::sso::util::SaSsoConsts;
use serde::Deserialize;
use serde_json::Value;

const JAVA_BASELINE: &str = "902886c2149261ccb53a9c982068b7ccd0990237";

#[derive(Deserialize)]
struct SsoGolden {
    source_commit: String,
    sso_client_auth_url: String,
    sso_client_is_http: bool,
    sso_client_is_slo: bool,
    sso_server_ticket_timeout: i64,
    sso_server_max_reg_client: i32,
    sso_api_check_ticket: String,
    sso_param_secretkey: String,
    sso_client_wildcard: String,
    sso_last_error_code: i32,
    sso_server_auth_url: String,
    sso_ticket_key: String,
    sso_ticket_index_key: String,
    sso_encoded_back_url: String,
}

struct GoldenSsoAuth;

impl SaSsoServerAuth for GoldenSsoAuth {
    fn login_device_id_by_token(
        &self,
        _: &str,
    ) -> Result<Option<String>, sa_token_sso::sso::exception::SaSsoException> {
        Ok(None)
    }

    fn token_timeout(&self, _: &str) -> Result<i64, sa_token_sso::sso::exception::SaSsoException> {
        Ok(0)
    }

    fn session_timeout(
        &self,
        _: &Value,
    ) -> Result<i64, sa_token_sso::sso::exception::SaSsoException> {
        Ok(0)
    }

    fn logout(
        &self,
        _: &Value,
        _: Option<String>,
    ) -> Result<(), sa_token_sso::sso::exception::SaSsoException> {
        Ok(())
    }
}

#[test]
fn sso_client_and_server_match_java_baseline() {
    let golden: SsoGolden = serde_json::from_str(include_str!("golden/sso.json"))
        .expect("Java sso golden must be valid JSON");
    assert_eq!(golden.source_commit, JAVA_BASELINE);

    let sso_client = SaSsoClientConfig::default();
    assert_eq!(sso_client.auth_url, golden.sso_client_auth_url);
    assert_eq!(sso_client.is_http, golden.sso_client_is_http);
    assert_eq!(sso_client.is_slo, golden.sso_client_is_slo);
    let sso_server = SaSsoServerConfig::default();
    assert_eq!(sso_server.ticket_timeout, golden.sso_server_ticket_timeout);
    assert_eq!(sso_server.max_reg_client, golden.sso_server_max_reg_client);
    assert_eq!(
        ApiName::default().sso_check_ticket,
        golden.sso_api_check_ticket
    );
    assert_eq!(ParamName::default().secret_key, golden.sso_param_secretkey);
    assert_eq!(SaSsoConsts::CLIENT_WILDCARD, golden.sso_client_wildcard);
    assert_eq!(SaSsoErrorCode::CODE_30024, golden.sso_last_error_code);

    let sso_dao: Arc<dyn SaTokenDao> = Arc::new(SaTokenDaoMemory::new());
    let sso_client_template = SaSsoClientTemplate::new(
        Arc::new(SaSsoClientConfig {
            client: Some("app-a".into()),
            server_url: Some("https://sso.example".into()),
            ..Default::default()
        }),
        Arc::new(SaSsoClientStrategy::default()),
        Arc::new(SaSignConfig::default()),
        Arc::clone(&sso_dao),
        "satoken",
        Arc::new(|_, _| Ok(())),
    )
    .expect("SSO client template");
    assert_eq!(
        sso_client_template
            .build_server_auth_url(
                "https://client.example/sso/login",
                Some("/home?a=1"),
            )
            .expect("SSO auth URL"),
        golden.sso_server_auth_url
    );

    let sso_server_template = SaSsoServerTemplate::new(
        Arc::new(SaSsoServerConfig::default()),
        Arc::new(SaSsoServerStrategy::default()),
        Arc::new(SaSignConfig::default()),
        sso_dao,
        Arc::new(GoldenSsoAuth),
        "satoken",
    )
    .expect("SSO server template");
    assert_eq!(sso_server_template.ticket_key("TICKET"), golden.sso_ticket_key);
    assert_eq!(
        sso_server_template.ticket_index_key("", &serde_json::json!(10001)),
        golden.sso_ticket_index_key
    );
    assert_eq!(
        sso_server_template.encode_back_param("https://client.example/sso/login?back=/home?a=1"),
        golden.sso_encoded_back_url
    );
}
