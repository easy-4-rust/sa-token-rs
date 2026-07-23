//! JSON serializer corresponding to Java `SaSerializerTemplateForJson`.

use serde_json::Value;

use crate::exception::{SaResult, SaTokenException};
use crate::serializer::SaSerializerTemplate;

/// JSON text serializer; byte conversion is deliberately unsupported like Java.
pub struct SaSerializerTemplateForJson;

impl SaSerializerTemplate for SaSerializerTemplateForJson {
    fn object_to_string(&self, object: Option<&Value>) -> SaResult<Option<String>> {
        object
            .map(serde_json::to_string)
            .transpose()
            .map_err(|error| SaTokenException::json_convert(error.to_string()))
    }

    fn string_to_object(&self, value: Option<&str>) -> SaResult<Option<Value>> {
        value
            .map(serde_json::from_str)
            .transpose()
            .map_err(|error| SaTokenException::json_convert(error.to_string()))
    }

    fn object_to_bytes(&self, _object: Option<&Value>) -> SaResult<Option<Vec<u8>>> {
        Err(SaTokenException::ApiDisabled)
    }

    fn bytes_to_object(&self, _bytes: Option<&[u8]>) -> SaResult<Option<Value>> {
        Err(SaTokenException::ApiDisabled)
    }
}
