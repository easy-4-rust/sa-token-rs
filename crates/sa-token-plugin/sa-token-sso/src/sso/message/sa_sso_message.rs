use crate::sso::error::SaSsoErrorCode;
use crate::sso::exception::SaSsoException;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

/// Structured message exchanged between an SSO server and client.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(transparent)]
pub struct SaSsoMessage {
    values: Map<String, Value>,
}

impl SaSsoMessage {
    /// Field containing the protocol message type.
    pub const MSG_TYPE: &'static str = "msgType";

    /// Creates an empty message.
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a message with its type initialized.
    pub fn with_type(message_type: impl Into<String>) -> Self {
        Self::new().set_type(message_type)
    }

    /// Creates a message from an existing JSON object.
    pub fn from_map(values: Map<String, Value>) -> Self {
        Self { values }
    }

    /// Returns the message type, when it is a string.
    pub fn get_type(&self) -> Option<&str> {
        self.values.get(Self::MSG_TYPE).and_then(Value::as_str)
    }

    /// Replaces the message type.
    pub fn set_type(mut self, message_type: impl Into<String>) -> Self {
        self.values.insert(
            Self::MSG_TYPE.to_owned(),
            Value::String(message_type.into()),
        );
        self
    }

    /// Verifies that the message has a type.
    ///
    /// # Errors
    ///
    /// Returns Java-compatible error code `30022` when `msgType` is absent or
    /// not a string.
    pub fn check_type(&self) -> Result<&str, SaSsoException> {
        self.get_type().ok_or_else(|| {
            SaSsoException::new(SaSsoErrorCode::CODE_30022, "SSO message is missing msgType")
        })
    }

    /// Adds or replaces a field and returns the message for fluent building.
    pub fn set(mut self, key: impl Into<String>, value: impl Into<Value>) -> Self {
        self.values.insert(key.into(), value.into());
        self
    }

    /// Adds or replaces a field in place.
    pub fn insert(&mut self, key: impl Into<String>, value: impl Into<Value>) -> Option<Value> {
        self.values.insert(key.into(), value.into())
    }

    /// Deletes a field.
    pub fn delete(&mut self, key: &str) -> Option<Value> {
        self.values.remove(key)
    }

    /// Returns a field value.
    pub fn get(&self, key: &str) -> Option<&Value> {
        self.values.get(key)
    }

    /// Returns a required field value.
    ///
    /// # Errors
    ///
    /// Returns Java-compatible error code `30024` when the field is absent.
    pub fn get_value_not_null(&self, key: &str) -> Result<&Value, SaSsoException> {
        self.get(key).ok_or_else(|| {
            SaSsoException::new(
                SaSsoErrorCode::CODE_30024,
                format!("SSO message is missing required field: {key}"),
            )
        })
    }

    /// Returns the underlying JSON object.
    pub fn as_map(&self) -> &Map<String, Value> {
        &self.values
    }
}
