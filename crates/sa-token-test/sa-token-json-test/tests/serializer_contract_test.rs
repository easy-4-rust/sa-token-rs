//! Contract coverage for every core serializer implementation.

use sa_token_core::exception::SaTokenException;
use sa_token_core::serializer::SaSerializerTemplate;
use sa_token_core::serializer::r#impl::{
    SaSerializerTemplateForJdk, SaSerializerTemplateForJdkUseBase64,
    SaSerializerTemplateForJdkUseHex, SaSerializerTemplateForJdkUseIso88591,
    SaSerializerTemplateForJson,
};
use serde_json::json;

fn assert_native_round_trip<T>(serializer: &T)
where
    T: SaSerializerTemplate,
{
    let value = json!({"id": 10001, "roles": ["admin", "auditor"]});
    let encoded = serializer
        .object_to_string(Some(&value))
        .expect("serialization must succeed")
        .expect("present input must remain present");
    let decoded = serializer
        .string_to_object(Some(&encoded))
        .expect("deserialization must succeed");
    assert_eq!(decoded, Some(value));
    assert_eq!(
        serializer.object_to_string(None).expect("None passthrough"),
        None
    );
}

#[test]
fn native_codec_adapters_round_trip_and_reject_malformed_input() {
    assert_native_round_trip(&SaSerializerTemplateForJdkUseBase64);
    assert_native_round_trip(&SaSerializerTemplateForJdkUseHex);
    assert_native_round_trip(&SaSerializerTemplateForJdkUseIso88591);

    assert!(
        SaSerializerTemplateForJdkUseBase64
            .string_to_object(Some("%%%"))
            .is_err()
    );
    assert!(
        SaSerializerTemplateForJdkUseHex
            .string_to_object(Some("xyz"))
            .is_err()
    );
    assert!(
        SaSerializerTemplateForJdkUseIso88591
            .string_to_bytes("你好")
            .is_err()
    );
}

#[test]
fn json_adapter_matches_java_disabled_byte_api() {
    let serializer = SaSerializerTemplateForJson;
    let value = json!({"ok": true});
    let encoded = serializer
        .object_to_string(Some(&value))
        .expect("JSON text serialization")
        .expect("present JSON");
    assert_eq!(
        serializer
            .string_to_object(Some(&encoded))
            .expect("JSON text deserialization"),
        Some(value.clone())
    );
    assert_eq!(
        serializer.object_to_bytes(Some(&value)),
        Err(SaTokenException::ApiDisabled)
    );
    assert_eq!(
        serializer.bytes_to_object(Some(b"{}")),
        Err(SaTokenException::ApiDisabled)
    );
    assert!(serializer.string_to_object(Some("{")).is_err());
}
