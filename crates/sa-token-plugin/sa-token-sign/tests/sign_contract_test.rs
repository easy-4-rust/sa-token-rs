use std::collections::HashMap;
use std::sync::Arc;

use sa_token_core::plugin::sa_token_plugin::SaTokenPlugin;
use sa_token_dao_memory::SaTokenDaoMemory;
use sa_token_sign::{
    SaCheckSign, SaCheckSignHandler, SaSignConfig, SaSignErrorCode, SaSignManager, SaSignMany,
    SaSignTemplate, SaTokenPluginForSign,
};

fn template(config: SaSignConfig) -> Arc<SaSignTemplate> {
    Arc::new(SaSignTemplate::new(
        Arc::new(config),
        Arc::new(SaTokenDaoMemory::new()),
        "satoken",
    ))
}

#[test]
fn canonical_signing_excludes_sign_and_supports_java_digests() {
    let signer = template(SaSignConfig::new("secret"));
    let params = HashMap::from([("b".into(), "2".into()), ("a".into(), "1".into())]);
    let signature = signer.create_sign(&params).expect("MD5 signature");
    assert_eq!(signature, "9f565ccd686cfa5dc3b06b3a89e4e3ad");
    let with_sign = HashMap::from([
        ("b".into(), "2".into()),
        ("a".into(), "1".into()),
        ("sign".into(), "ignored".into()),
    ]);
    assert_eq!(
        signer.create_sign(&with_sign).expect("same signature"),
        signature
    );
    assert_eq!(
        template(SaSignConfig::new("secret").with_digest_algo("sha256"))
            .create_sign(&params)
            .expect("SHA-256 signature")
            .len(),
        64
    );
}

#[test]
fn timestamp_nonce_and_signature_are_checked_in_java_order() {
    let signer = template(SaSignConfig::new("secret").with_timestamp_disparity(60_000));
    let mut params = HashMap::from([("data".into(), "value".into())]);
    signer.add_sign_params(&mut params).expect("signed params");
    signer
        .check_param_map(&params)
        .expect("first request succeeds");
    assert!(
        signer.check_param_map(&params).is_err(),
        "nonce replay must fail"
    );
    let stale = chrono::Utc::now().timestamp_millis() - 61_000;
    assert_eq!(
        signer
            .check_timestamp(stale)
            .expect_err("stale timestamp")
            .code,
        SaSignErrorCode::CODE_12203
    );
}

#[test]
fn named_config_annotation_and_plugin_lifecycle_are_isolated() {
    let default = template(SaSignConfig::new("default"));
    let manager = Arc::new(SaSignManager::new(Arc::clone(&default)));
    manager
        .register("app-1", Arc::new(SaSignConfig::new("secret")))
        .expect("register");
    let dao = Arc::new(SaTokenDaoMemory::new());
    let factory =
        Arc::new(move |config| Arc::new(SaSignTemplate::new(config, dao.clone(), "satoken")));
    let many = Arc::new(SaSignMany::new(manager, factory));
    let missing = match many.get_sign_template("missing") {
        Ok(_) => panic!("missing app must fail"),
        Err(error) => error,
    };
    assert_eq!(missing.code, SaSignErrorCode::CODE_12211);
    let handler = Arc::new(SaCheckSignHandler::new(many));
    let plugin = SaTokenPluginForSign::new(Arc::clone(&handler));
    assert!(plugin.handler().is_none());
    plugin.install();
    assert!(plugin.handler().is_some());
    plugin.destroy();
    let metadata = SaCheckSign {
        app_id: "#{appid}".into(),
        verify_params: vec!["data".into()],
    };
    assert_eq!(metadata.app_id, "#{appid}");
}

#[test]
fn missing_secret_and_bad_signature_have_detailed_codes() {
    let empty = template(SaSignConfig::default());
    assert_eq!(
        empty
            .create_sign(&HashMap::new())
            .expect_err("missing secret")
            .code,
        SaSignErrorCode::CODE_12201
    );
    let signer = template(SaSignConfig::new("secret"));
    assert_eq!(
        signer
            .check_sign(&HashMap::new(), "wrong")
            .expect_err("bad sign")
            .code,
        SaSignErrorCode::CODE_12202
    );
}
