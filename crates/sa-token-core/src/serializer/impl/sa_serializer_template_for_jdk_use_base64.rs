//! Native object serialization with standard Base64 text encoding.

use serde_json::Value;

use super::sa_serializer_template_for_jdk::SaSerializerTemplateForJdk;
use crate::exception::{SaResult, SaTokenException};
use crate::secure::sa_base64_util::SaBase64Util;
use crate::serializer::SaSerializerTemplate;

/// Counterpart of Java `SaSerializerTemplateForJdkUseBase64`.
pub struct SaSerializerTemplateForJdkUseBase64;

impl SaSerializerTemplateForJdk for SaSerializerTemplateForJdkUseBase64 {
    fn bytes_to_string(&self, bytes: &[u8]) -> SaResult<String> {
        Ok(SaBase64Util::encode(bytes))
    }

    fn string_to_bytes(&self, value: &str) -> SaResult<Vec<u8>> {
        SaBase64Util::decode(value)
            .map_err(|error| SaTokenException::json_convert(error.to_string()))
    }
}

impl SaSerializerTemplate for SaSerializerTemplateForJdkUseBase64 {
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
