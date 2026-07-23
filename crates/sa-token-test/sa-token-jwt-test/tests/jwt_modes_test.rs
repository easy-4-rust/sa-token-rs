//! 对应 Java：`sa-token-jwt-test`
//! - `JwtForSimpleTest.java`
//! - `JwtForMixinTest.java`
//! - `JwtForStatelessTest.java`

use std::collections::HashMap;

use sa_token_jwt::{
    SaJwtErrorCode, SaJwtTemplate, StpLogicJwtForMixin, StpLogicJwtForSimple,
    StpLogicJwtForStateless,
};
use serde_json::{Value, json};

const SECRET: &str = "test-secret-key-with-enough-entropy";

/// 对应 `JwtForSimpleTest`：claims / 毫秒过期 / HS256。
#[test]
fn jwt_for_simple_claims_and_timeout() {
    let token = SaJwtTemplate
        .create_token_full(
            "login",
            json!(10001),
            "web",
            60,
            HashMap::from([("role".into(), json!("admin"))]),
            SECRET,
        )
        .expect("create");
    let payload = SaJwtTemplate
        .get_payloads(&token, "login", SECRET)
        .expect("parse");
    assert_eq!(payload[SaJwtTemplate::LOGIN_ID], json!(10001));
    assert_eq!(payload[SaJwtTemplate::DEVICE_TYPE], json!("web"));
    assert!((59..=60).contains(&SaJwtTemplate.get_timeout(&token, "login", SECRET)));
}

/// 对应 `JwtForSimpleTest` 错误码段：30201 等。
#[test]
fn jwt_validation_error_codes() {
    let err = SaJwtTemplate
        .parse_token("not-a-jwt", "login", SECRET, true)
        .expect_err("malformed");
    assert_eq!(err.code(), SaJwtErrorCode::CODE_30201);
}

/// 对应三种运行模式类型可构造（Simple / Mixin / Stateless）。
#[test]
fn jwt_logic_modes_constructible() {
    let _simple = StpLogicJwtForSimple::new("login", SECRET);
    let _mixin = StpLogicJwtForMixin::new("login", SECRET);
    let _stateless = StpLogicJwtForStateless::new("login", SECRET);
    let _: Value = json!({});
}
