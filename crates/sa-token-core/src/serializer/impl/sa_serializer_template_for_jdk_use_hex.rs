//! Native object serialization with hexadecimal text encoding.

use serde_json::Value;

use super::sa_serializer_template_for_jdk::SaSerializerTemplateForJdk;
use crate::exception::{SaResult, SaTokenException};
use crate::serializer::SaSerializerTemplate;
use crate::util::sa_hex_util;

/// Counterpart of Java `SaSerializerTemplateForJdkUseHex`.
pub struct SaSerializerTemplateForJdkUseHex;

impl SaSerializerTemplateForJdk for SaSerializerTemplateForJdkUseHex {
    fn bytes_to_string(&self, bytes: &[u8]) -> SaResult<String> {
        Ok(sa_hex_util::encode(bytes))
    }

    fn string_to_bytes(&self, value: &str) -> SaResult<Vec<u8>> {
        sa_hex_util::decode(value).map_err(SaTokenException::json_convert)
    }
}

impl SaSerializerTemplate for SaSerializerTemplateForJdkUseHex {
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
