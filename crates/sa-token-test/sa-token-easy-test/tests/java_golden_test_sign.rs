//! Per-domain golden values for the SaSign MD5 signatures.
//!
//! Fixture is sliced from the master `golden/core.json` by
//! `cargo xtask golden-split`. Keep keys field list below in sync with
//! `xtask/src/main.rs::domain_keys(DOMAIN_SIGN)`.

use std::collections::HashMap;
use std::sync::Arc;

use sa_token_dao_memory::SaTokenDaoMemory;
use sa_token_sign::{SaSignConfig, SaSignTemplate};
use serde::Deserialize;

const JAVA_BASELINE: &str = "902886c2149261ccb53a9c982068b7ccd0990237";

#[derive(Deserialize)]
struct SignGolden {
    source_commit: String,
    sign_default_timestamp_disparity: i64,
    sign_default_digest: String,
    sign_md5: String,
}

#[test]
fn sign_md5_signature_matches_java_baseline() {
    let golden: SignGolden = serde_json::from_str(include_str!("golden/sign.json"))
        .expect("Java sign golden must be valid JSON");
    assert_eq!(golden.source_commit, JAVA_BASELINE);

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
            .create_sign(&HashMap::from([
                ("b".into(), "2".into()),
                ("a".into(), "1".into()),
            ]))
            .expect("Java-compatible MD5 signature"),
        golden.sign_md5
    );
}
