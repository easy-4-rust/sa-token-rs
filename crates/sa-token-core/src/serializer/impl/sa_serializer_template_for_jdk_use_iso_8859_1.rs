//! Native object serialization with ISO-8859-1 text encoding.

use serde_json::Value;

use super::sa_serializer_template_for_jdk::SaSerializerTemplateForJdk;
use crate::exception::{SaResult, SaTokenException};
use crate::serializer::SaSerializerTemplate;

/// Counterpart of Java `SaSerializerTemplateForJdkUseISO_8859_1`.
pub struct SaSerializerTemplateForJdkUseIso88591;

impl SaSerializerTemplateForJdk for SaSerializerTemplateForJdkUseIso88591 {
    fn bytes_to_string(&self, bytes: &[u8]) -> SaResult<String> {
        Ok(bytes.iter().map(|byte| char::from(*byte)).collect())
    }

    fn string_to_bytes(&self, value: &str) -> SaResult<Vec<u8>> {
        value
            .chars()
            .map(|character| {
                u8::try_from(u32::from(character)).map_err(|_| {
                    SaTokenException::json_convert(format!("字符 {character:?} 不属于 ISO-8859-1"))
                })
            })
            .collect()
    }
}

impl SaSerializerTemplate for SaSerializerTemplateForJdkUseIso88591 {
    fn object_to_string(&self, object: Option<&Value>) -> SaResult<Option<String>> {
        self.native_object_to_string(object)
    }

    fn string_to_object(&self, value: Option<&str>) -> SaResult<Option<Value>> {
        self.native_string_to_object(value)
    }

    fn object_to_bytes(&self, object: Option<&Value>) -> SaResult<Option<Vec<u8>>> {
        self.native_object_to_bytes(object)
    }

    fn bytes_to_object(&self, bytes: Option<&[u8]>) -> SaResult<Option<Value>> {
        self.native_bytes_to_object(bytes)
    }
}
