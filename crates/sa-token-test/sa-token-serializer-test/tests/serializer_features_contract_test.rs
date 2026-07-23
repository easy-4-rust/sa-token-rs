//! Contract tests for custom Base64 serializer features.

use sa_token_core::plugin::sa_token_plugin::SaTokenPlugin;
use sa_token_core::serializer::SaSerializerTemplate;
use sa_token_core::serializer::r#impl::SaSerializerTemplateForJdk;
use sa_token_serializer_features::{
    SaSerializerForBase64UseCustomCharacters, SaSerializerForBase64UseEmoji,
    SaSerializerForBase64UsePeriodicTable, SaSerializerForBase64UseSpecialSymbols,
    SaSerializerForBase64UseTianGan, SaTokenPluginForSerializerFeatures,
};
use serde_json::json;

fn assert_codec_round_trip(serializer: &impl SaSerializerTemplateForJdk) {
    for bytes in [b"".as_slice(), b"S", b"Sa", b"SaToken", &[0, 1, 254, 255]] {
        let encoded = serializer
            .bytes_to_string(bytes)
            .expect("valid bytes must encode");
        assert_eq!(
            serializer
                .string_to_bytes(&encoded)
                .expect("generated text must decode"),
            bytes
        );
    }
}

#[test]
fn all_built_in_alphabets_round_trip_bytes() {
    assert_codec_round_trip(&SaSerializerForBase64UseEmoji::default());
    assert_codec_round_trip(&SaSerializerForBase64UsePeriodicTable::default());
    assert_codec_round_trip(&SaSerializerForBase64UseSpecialSymbols::default());
    assert_codec_round_trip(&SaSerializerForBase64UseTianGan::default());
}

#[test]
fn custom_alphabet_validates_configuration_and_input() {
    let alphabet = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let serializer = SaSerializerForBase64UseCustomCharacters::new(alphabet, '=')
        .expect("standard alphabet is valid");
    assert_codec_round_trip(&serializer);
    assert!(SaSerializerForBase64UseCustomCharacters::new("short", '=').is_err());
    assert!(SaSerializerForBase64UseCustomCharacters::new(alphabet, 'A').is_err());
    assert!(SaSerializerForBase64UseCustomCharacters::new(&"A".repeat(64), '=').is_err());
    assert!(serializer.string_to_bytes("bad").is_err());
    assert!(serializer.string_to_bytes("A===").is_err());
    assert!(serializer.string_to_bytes("AA=A").is_err());
}

#[test]
fn serializer_port_preserves_objects_and_explicit_errors() {
    let serializer = SaSerializerForBase64UseTianGan::default();
    let object = json!({"login_id": 10001, "roles": ["admin"]});
    let encoded = serializer
        .object_to_string(Some(&object))
        .expect("object serialization")
        .expect("present input");
    assert_eq!(
        serializer
            .string_to_object(Some(&encoded))
            .expect("object deserialization"),
        Some(object)
    );
    assert_eq!(
        serializer.object_to_string(None).expect("None passthrough"),
        None
    );
    assert!(serializer.string_to_object(Some("未知字符")).is_err());
}

#[test]
fn plugin_install_is_an_intentional_no_op() {
    let plugin = SaTokenPluginForSerializerFeatures;
    plugin.install();
    plugin.destroy();
    assert!(plugin.as_any().is::<SaTokenPluginForSerializerFeatures>());
}
