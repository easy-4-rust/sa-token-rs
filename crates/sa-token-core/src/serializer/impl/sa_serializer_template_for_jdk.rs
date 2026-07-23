//! Native Rust object serialization with a pluggable text codec.

use serde_json::Value;

use crate::exception::{SaResult, SaTokenException};

/// Shared behavior corresponding to Java `SaSerializerTemplateForJdk`.
///
/// Java uses `ObjectOutputStream`; Rust uses Serde JSON as its portable native
/// object representation and retains the Java class's pluggable byte/text codec.
pub trait SaSerializerTemplateForJdk {
    /// Encodes bytes into the adapter's text representation.
    fn bytes_to_string(&self, bytes: &[u8]) -> SaResult<String>;

    /// Decodes the adapter's text representation into bytes.
    fn string_to_bytes(&self, value: &str) -> SaResult<Vec<u8>>;

    /// Serializes an optional object to native bytes.
    fn native_object_to_bytes(&self, object: Option<&Value>) -> SaResult<Option<Vec<u8>>> {
        object
            .map(serde_json::to_vec)
            .transpose()
            .map_err(|error| SaTokenException::json_convert(error.to_string()))
    }

    /// Deserializes optional native bytes.
    fn native_bytes_to_object(&self, bytes: Option<&[u8]>) -> SaResult<Option<Value>> {
        bytes
            .map(serde_json::from_slice)
            .transpose()
            .map_err(|error| SaTokenException::json_convert(error.to_string()))
    }

    /// Serializes an optional object through the configured text codec.
    fn native_object_to_string(&self, object: Option<&Value>) -> SaResult<Option<String>> {
        self.native_object_to_bytes(object)?
            .map(|bytes| self.bytes_to_string(&bytes))
            .transpose()
    }

    /// Deserializes optional codec text into an object.
    fn native_string_to_object(&self, value: Option<&str>) -> SaResult<Option<Value>> {
        match value {
            Some(value) => {
                let bytes = self.string_to_bytes(value)?;
                self.native_bytes_to_object(Some(&bytes))
            }
            None => Ok(None),
        }
    }
}
