//! JWT mode, claim, expiry, and error-code contracts.

use std::collections::HashMap;

use sa_token_jwt::{
    SaJwtErrorCode, SaJwtTemplate, StpLogicJwtForMixin, StpLogicJwtForSimple,
    StpLogicJwtForStateless,
};
use serde_json::{Value, json};

const SECRET: &str = "test-secret-key-with-enough-entropy";

#[test]
fn template_uses_java_claims_and_millisecond_expiry() {
    let token = SaJwtTemplate
        .create_token_full(
            "login",
            json!(10001),
            "web",
            60,
            HashMap::from([("role".into(), json!("admin"))]),
            SECRET,
        )
        .expect("full JWT creation");
    let payload = SaJwtTemplate
        .get_payloads(&token, "login", SECRET)
        .expect("full JWT parsing");
    assert_eq!(payload[SaJwtTemplate::LOGIN_ID], json!(10001));
    assert_eq!(payload[SaJwtTemplate::DEVICE_TYPE], json!("web"));
    assert_eq!(payload["role"], json!("admin"));
    assert_eq!(
        payload[SaJwtTemplate::RN_STR].as_str().map(str::len),
        Some(32)
    );
    assert!((59..=60).contains(&SaJwtTemplate.get_timeout(&token, "login", SECRET)));
}

#[test]
fn validation_errors_preserve_java_codes() {
    let token = SaJwtTemplate
        .create_token_full("login", json!(1), "web", 60, HashMap::new(), SECRET)
        .expect("JWT creation");
    assert_eq!(
        SaJwtTemplate
            .parse_token("not-a-jwt", "login", SECRET, true)
            .expect_err("malformed JWT")
            .code(),
        SaJwtErrorCode::CODE_30201
    );
    assert_eq!(
        SaJwtTemplate
            .parse_token(&token, "login", "wrong-secret", true)
            .expect_err("invalid signature")
            .code(),
        SaJwtErrorCode::CODE_30202
    );
    assert_eq!(
        SaJwtTemplate
            .parse_token(&token, "admin", SECRET, true)
            .expect_err("invalid login type")
            .code(),
        SaJwtErrorCode::CODE_30203
    );
    assert_eq!(
        SaJwtTemplate
            .create_token_full("login", json!(1), "web", -2, HashMap::new(), SECRET)
            .and_then(|token| SaJwtTemplate.parse_token(&token, "login", SECRET, true))
            .expect_err("expired JWT")
            .code(),
        SaJwtErrorCode::CODE_30204
    );
    assert_eq!(
        SaJwtTemplate
            .create_token("login", json!(1), HashMap::new(), "")
            .expect_err("missing secret")
            .code(),
        SaJwtErrorCode::CODE_30205
    );
    assert_eq!(
        SaJwtTemplate
            .create_token("login", Value::Null, HashMap::new(), SECRET)
            .expect_err("missing login id")
            .code(),
        SaJwtErrorCode::CODE_30206
    );
}

#[test]
fn simple_mode_has_no_expiry_and_never_reuses_tokens() {
    let logic = StpLogicJwtForSimple::new("login", SECRET);
    let first = logic
        .create_token_value(
            json!(10001),
            HashMap::from([("tenant".into(), json!("t1"))]),
        )
        .expect("simple JWT");
    let second = logic
        .create_token_value(json!(10001), HashMap::new())
        .expect("second simple JWT");
    assert_ne!(first, second);
    assert_eq!(
        logic.get_extra(&first, "tenant").expect("extra"),
        Some(json!("t1"))
    );
    assert!(!logic.is_support_share_token());
    assert!(logic.is_support_extra());
}

#[test]
fn mixin_and_stateless_modes_keep_distinct_storage_boundaries() {
    let mixin = StpLogicJwtForMixin::new("login", SECRET);
    let mixin_token = mixin
        .create_token_value(json!(10001), "app", -1, HashMap::new())
        .expect("mixin JWT");
    assert_eq!(
        mixin.get_login_id(&mixin_token).expect("mixin identity"),
        json!(10001)
    );
    assert_eq!(mixin.get_token_timeout(&mixin_token), -1);
    assert!(mixin.supports_token_session());

    let stateless = StpLogicJwtForStateless::new("login", SECRET);
    let token = stateless
        .create_login_session(json!(10002), "web", 60, HashMap::new())
        .expect("stateless JWT");
    assert_eq!(
        stateless.get_login_id(&token).expect("identity"),
        json!(10002)
    );
    assert_eq!(
        stateless.get_login_device_type(&token).expect("device"),
        Some("web".into())
    );
    assert!(!stateless.supports_persistent_dao());
}
