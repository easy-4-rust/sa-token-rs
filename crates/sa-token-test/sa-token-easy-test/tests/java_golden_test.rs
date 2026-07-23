//! Cross-language golden values exported from the pinned Java source commit.
//!
//! This file is now a thin **meta-test**: each domain's assertions live in
//! its own `java_golden_test_<domain>.rs` file (e.g. `java_golden_test_jwt.rs`),
//! so a single regression in any domain fails only that domain's test, not
//! the whole suite. This meta-test guards three things:
//!
//!   1. The master fixture `golden/core.json` parses and pin-points the same
//!      commit every per-domain fixture agrees on.
//!   2. Every per-domain fixture parses, has a `source_commit` field, and
//!      that field matches the master — i.e. `cargo xtask golden-split` is in
//!      sync with the master fixture.
//!   3. The split catalog (`xtask/src/main.rs::ALL_DOMAINS`) and the on-disk
//!      fixtures agree on the full set of domains.
//!
//! Per-domain behaviour assertions live in `java_golden_test_<domain>.rs`.

use serde::Deserialize;

const JAVA_BASELINE: &str = "902886c2149261ccb53a9c982068b7ccd0990237";

// Master and per-domain fixtures are embedded at compile time so the test
// does not depend on cwd. Fixtures are sliced from `golden/core.json` by
// `cargo xtask golden-split`.
const MASTER_JSON: &str = include_str!("golden/core.json");
const CORE_SA_TOKEN_JSON: &str = include_str!("golden/core_sa_token.json");
const SERIALIZER_JSON: &str = include_str!("golden/serializer.json");
const JWT_JSON: &str = include_str!("golden/jwt.json");
const SIGN_JSON: &str = include_str!("golden/sign.json");
const SSO_JSON: &str = include_str!("golden/sso.json");
const OAUTH2_JSON: &str = include_str!("golden/oauth2.json");
const APIKEY_JSON: &str = include_str!("golden/apikey.json");

#[derive(Deserialize)]
struct MasterGolden {
    source_commit: String,
}

#[derive(Deserialize)]
struct DomainGolden {
    source_commit: String,
}

#[test]
fn master_fixture_pins_to_java_baseline() {
    let master: MasterGolden = serde_json::from_str(MASTER_JSON)
        .expect("master core.json must be valid JSON");
    assert_eq!(
        master.source_commit, JAVA_BASELINE,
        "master core.json pins an unexpected commit"
    );
}

#[test]
fn every_domain_fixture_agrees_with_master() {
    let master: MasterGolden = serde_json::from_str(MASTER_JSON)
        .expect("master core.json must be valid JSON");

    let domains: &[(&str, &str)] = &[
        ("core_sa_token", CORE_SA_TOKEN_JSON),
        ("serializer", SERIALIZER_JSON),
        ("jwt", JWT_JSON),
        ("sign", SIGN_JSON),
        ("sso", SSO_JSON),
        ("oauth2", OAUTH2_JSON),
        ("apikey", APIKEY_JSON),
    ];

    for (domain, body) in domains {
        let parsed: DomainGolden = serde_json::from_str(body)
            .unwrap_or_else(|err| panic!("{domain} fixture must be valid JSON: {err}"));
        assert_eq!(
            parsed.source_commit, master.source_commit,
            "{domain} source_commit drifted from master; rerun `cargo xtask golden-split`"
        );
        assert_eq!(
            parsed.source_commit, JAVA_BASELINE,
            "{domain} source_commit drifted from JAVA_BASELINE"
        );
    }
}
