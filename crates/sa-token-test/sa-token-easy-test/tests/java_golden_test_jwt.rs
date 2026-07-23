//! Per-domain golden values for JWT HS256 round-trip.
//!
//! Fixture is sliced from the master `golden/core.json` by
//! `cargo xtask golden-split`. Keep keys field list below in sync with
//! `xtask/src/main.rs::domain_keys(DOMAIN_JWT)`.

use sa_token_jwt::SaJwtTemplate;
use serde::Deserialize;

const JAVA_BASELINE: &str = "902886c2149261ccb53a9c982068b7ccd0990237";

#[derive(Deserialize)]
struct JwtGolden {
    source_commit: String,
    jwt_hs256_token: String,
}

#[test]
fn jwt_hs256_token_is_verifiable_and_carries_java_payloads() {
    let golden: JwtGolden = serde_json::from_str(include_str!("golden/jwt.json"))
        .expect("Java jwt golden must be valid JSON");
    assert_eq!(golden.source_commit, JAVA_BASELINE);

    let java_jwt = SaJwtTemplate
        .get_payloads(&golden.jwt_hs256_token, "login", "java-golden-secret")
        .expect("Rust must verify the Java HS256 token");
    assert_eq!(java_jwt[SaJwtTemplate::LOGIN_ID], serde_json::json!(10001));
    assert_eq!(
        java_jwt[SaJwtTemplate::DEVICE_TYPE],
        serde_json::json!("web")
    );
    assert_eq!(java_jwt[SaJwtTemplate::EFF], serde_json::json!(-1));
}
