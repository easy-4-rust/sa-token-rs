//! Per-domain golden values for the JDK + Base64 family of serializers.
//!
//! 7 keys: JDK-Base64 / JDK-Hex / JDK-ISO-8859-1 / 4× Base64-with-charset.
//! Fixture is sliced from the master `golden/core.json` by
//! `cargo xtask golden-split`. Keep keys field list below in sync with
//! `xtask/src/main.rs::domain_keys(DOMAIN_SERIALIZER)`.

use sa_token_core::serializer::r#impl::{
    SaSerializerTemplateForJdkUseBase64, SaSerializerTemplateForJdkUseHex,
    SaSerializerTemplateForJdkUseIso88591,
};
use sa_token_core::serializer::SaSerializerTemplateForJdk;
use sa_token_serializer_features::{
    SaSerializerForBase64UseEmoji, SaSerializerForBase64UsePeriodicTable,
    SaSerializerForBase64UseSpecialSymbols, SaSerializerForBase64UseTianGan,
};
use serde::Deserialize;

const JAVA_BASELINE: &str = "902886c2149261ccb53a9c982068b7ccd0990237";

#[derive(Deserialize)]
struct SerializerGolden {
    source_commit: String,
    serializer_base64: String,
    serializer_hex: String,
    serializer_iso_8859_1: String,
    serializer_emoji: String,
    serializer_periodic_table: String,
    serializer_special_symbols: String,
    serializer_tian_gan: String,
}

#[test]
fn serializer_outputs_match_java_baseline() {
    let golden: SerializerGolden = serde_json::from_str(include_str!("golden/serializer.json"))
        .expect("Java serializer golden must be valid JSON");
    assert_eq!(golden.source_commit, JAVA_BASELINE);

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
}
