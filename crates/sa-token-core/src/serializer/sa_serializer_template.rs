//! Serializer port corresponding to Java `SaSerializerTemplate`.

use serde_json::Value;

use crate::exception::{SaResult, SaTokenException};

/// Object serialization boundary used by persistence adapters.
///
/// Missing input remains `None`; malformed data and unsupported conversions are
/// returned as explicit errors and are never collapsed into missing values.
pub trait SaSerializerTemplate: Send + Sync + 'static {
    /// Serializes an optional object into text.
    ///
    /// # Errors
    /// Returns [`SaTokenException::JsonConvert`] when serialization fails.
    fn object_to_string(&self, object: Option<&Value>) -> SaResult<Option<String>>;

    /// Deserializes optional text into an object.
    ///
    /// # Errors
    /// Returns [`SaTokenException::JsonConvert`] for malformed input.
    fn string_to_object(&self, value: Option<&str>) -> SaResult<Option<Value>>;

    /// Deserializes optional text into a concrete type.
    ///
    /// # Errors
    /// Returns [`SaTokenException::JsonConvert`] for malformed input or a type mismatch.
    fn string_to_object_typed<T>(&self, value: Option<&str>) -> SaResult<Option<T>>
    where
        T: serde::de::DeserializeOwned,
        Self: Sized,
    {
        self.string_to_object(value)?
            .map(serde_json::from_value)
            .transpose()
            .map_err(|error| SaTokenException::json_convert(error.to_string()))
    }

    /// Serializes an optional object into bytes.
    ///
    /// # Errors
    /// Returns an explicit conversion or unsupported-operation error.
    fn object_to_bytes(&self, object: Option<&Value>) -> SaResult<Option<Vec<u8>>>;

    /// Deserializes optional bytes into an object.
    ///
    /// # Errors
    /// Returns an explicit conversion or unsupported-operation error.
    fn bytes_to_object(&self, bytes: Option<&[u8]>) -> SaResult<Option<Value>>;
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::SaSerializerTemplate;
    use crate::serializer::r#impl::SaSerializerTemplateForJson;

    #[test]
    fn json_text_round_trip_preserves_null_and_errors() {
        let serializer = SaSerializerTemplateForJson;
        let value = json!({"a": 1, "b": "x"});
        let encoded = serializer
            .object_to_string(Some(&value))
            .expect("JSON serialization must succeed")
            .expect("present input must remain present");
        let decoded = serializer
            .string_to_object(Some(&encoded))
            .expect("JSON deserialization must succeed");
        assert_eq!(decoded, Some(value));
        assert_eq!(
            serializer.object_to_string(None).expect("null passthrough"),
            None
        );
        assert!(serializer.string_to_object(Some("{")).is_err());
    }
}
