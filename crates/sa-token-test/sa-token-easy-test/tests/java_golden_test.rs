//! Cross-language golden values exported from the pinned Java source commit.

use std::sync::Arc;

use sa_token_apikey::apikey::error::SaApiKeyErrorCode;
use sa_token_apikey::apikey::loader::SaApiKeyDataLoaderDefaultImpl;
use sa_token_apikey::{SaApiKeyConfig, SaApiKeyTemplate};
use sa_token_core::config::sa_token_config::SaTokenConfig;
use sa_token_core::serializer::r#impl::{
    SaSerializerTemplateForJdk, SaSerializerTemplateForJdkUseBase64,
    SaSerializerTemplateForJdkUseHex, SaSerializerTemplateForJdkUseIso88591,
};
use sa_token_core::stp::stp_logic::StpLogic;
use sa_token_dao_memory::SaTokenDaoMemory;
use sa_token_jwt::SaJwtTemplate;
use sa_token_oauth2::oauth2::config::SaOAuth2OidcConfig;
use sa_token_oauth2::oauth2::consts::{GrantType, SaOAuth2Api, SaOAuth2Consts};
use sa_token_oauth2::oauth2::dao::SaOAuth2Dao;
use sa_token_oauth2::oauth2::data::loader::{SaOAuth2DataLoader, SaOAuth2DataLoaderDefaultImpl};
use sa_token_oauth2::oauth2::error::SaOAuth2ErrorCode;
use sa_token_serializer_features::{
    SaSerializerForBase64UseEmoji, SaSerializerForBase64UsePeriodicTable,
    SaSerializerForBase64UseSpecialSymbols, SaSerializerForBase64UseTianGan,
};
use sa_token_sign::{SaSignConfig, SaSignTemplate};
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
struct CoreGolden {
    source_commit: String,
    token_name: String,
    timeout: i64,
    active_timeout: i64,
    token_key: String,
    session_key: String,
    token_session_key: String,
    last_active_key: String,
    switch_key: String,
    disable_key: String,
    disable_service_key: String,
    safe_key: String,
    safe_service_key: String,
    serializer_base64: String,
    serializer_hex: String,
    serializer_iso_8859_1: String,
    serializer_emoji: String,
    serializer_periodic_table: String,
    serializer_special_symbols: String,
    serializer_tian_gan: String,
    jwt_hs256_token: String,
    sign_default_timestamp_disparity: i64,
    sign_default_digest: String,
    sign_md5: String,
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
    api_key_prefix: String,
    api_key_timeout: i64,
    api_key_record_index: bool,
    api_key_save_key: String,
    api_key_invalid_code: i32,
    api_key_scope_code: i32,
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
fn core_keys_and_defaults_match_java_baseline() {
    let golden: CoreGolden = serde_json::from_str(include_str!("golden/core.json"))
        .expect("Java core golden must be valid JSON");
    assert_eq!(golden.source_commit, JAVA_BASELINE);

    let config = Arc::new(SaTokenConfig::default());
    assert_eq!(config.token_name, golden.token_name);
    assert_eq!(config.timeout, golden.timeout);
    assert_eq!(config.active_timeout, golden.active_timeout);

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

    let bytes = b"SaToken";
    assert_eq!(
        SaSerializerTemplateForJdkUseBase64
            .bytes_to_string(bytes)
            .expect("Base64 encoding"),
        golden.serializer_base64
    );
    assert_eq!(
        SaSerializerTemplateForJdkUseHex
            .bytes_to_string(bytes)
            .expect("hex encoding"),
        golden.serializer_hex
    );
    assert_eq!(
        SaSerializerTemplateForJdkUseIso88591
            .bytes_to_string(bytes)
            .expect("ISO-8859-1 encoding"),
        golden.serializer_iso_8859_1
    );
    assert_eq!(
        SaSerializerForBase64UseEmoji::default()
            .bytes_to_string(bytes)
            .expect("emoji encoding"),
        golden.serializer_emoji
    );
    assert_eq!(
        SaSerializerForBase64UsePeriodicTable::default()
            .bytes_to_string(bytes)
            .expect("periodic-table encoding"),
        golden.serializer_periodic_table
    );
    assert_eq!(
        SaSerializerForBase64UseSpecialSymbols::default()
            .bytes_to_string(bytes)
            .expect("special-symbol encoding"),
        golden.serializer_special_symbols
    );
    assert_eq!(
        SaSerializerForBase64UseTianGan::default()
            .bytes_to_string(bytes)
            .expect("heavenly-stems encoding"),
        golden.serializer_tian_gan
    );
    let java_jwt = SaJwtTemplate
        .get_payloads(&golden.jwt_hs256_token, "login", "java-golden-secret")
        .expect("Rust must verify the Java HS256 token");
    assert_eq!(java_jwt[SaJwtTemplate::LOGIN_ID], serde_json::json!(10001));
    assert_eq!(
        java_jwt[SaJwtTemplate::DEVICE_TYPE],
        serde_json::json!("web")
    );
    assert_eq!(java_jwt[SaJwtTemplate::EFF], serde_json::json!(-1));
    let sign_config = SaSignConfig::default();
    assert_eq!(
        sign_config.timestamp_disparity,
        golden.sign_default_timestamp_disparity
    );
    assert_eq!(sign_config.digest_algo, golden.sign_default_digest);
    let sign_template = SaSignTemplate::new(
        Arc::new(SaSignConfig::new("secret")),
        Arc::new(SaTokenDaoMemory::new()),
        "satoken",
    );
    assert_eq!(
        sign_template
            .create_sign(&std::collections::HashMap::from([
                ("b".into(), "2".into()),
                ("a".into(), "1".into()),
            ]))
            .expect("Java-compatible MD5 signature"),
        golden.sign_md5
    );
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
    let sso_dao: Arc<dyn sa_token_core::dao::sa_token_dao::SaTokenDao> =
        Arc::new(SaTokenDaoMemory::new());
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
            .build_server_auth_url("https://client.example/sso/login", Some("/home?a=1"),)
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
    assert_eq!(
        sso_server_template.ticket_key("TICKET"),
        golden.sso_ticket_key
    );
    assert_eq!(
        sso_server_template.ticket_index_key("", &serde_json::json!(10001)),
        golden.sso_ticket_index_key
    );
    assert_eq!(
        sso_server_template.encode_back_param("https://client.example/sso/login?back=/home?a=1"),
        golden.sso_encoded_back_url
    );
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
