//! Common typed reads for application, session, and request storage.

use serde::de::DeserializeOwned;
use serde_json::Value;

use crate::exception::{SaResult, SaTokenException};

/// Common value-reading contract.
pub trait SaGetValueInterface: Send + Sync {
    /// Reads the raw JSON value.
    ///
    /// # Errors
    ///
    /// Returns a storage or deserialization failure.
    fn get(&self, key: &str) -> SaResult<Option<Value>>;

    /// Returns whether the raw value is Java-compatible null or empty text.
    fn value_is_null(value: Option<&Value>) -> bool {
        match value {
            None | Some(Value::Null) => true,
            Some(Value::String(text)) => text.is_empty(),
            Some(_) => false,
        }
    }

    /// Reads and converts a value, falling back when it is absent or empty.
    ///
    /// # Errors
    ///
    /// Returns a storage or conversion failure.
    fn get_with_default<T>(&self, key: &str, default_value: T) -> SaResult<T>
    where
        T: DeserializeOwned,
        Self: Sized,
    {
        match self.get(key)? {
            Some(value) if !Self::value_is_null(Some(&value)) => convert_value(value),
            _ => Ok(default_value),
        }
    }

    /// Reads a value using Java's `String.valueOf`-style representation.
    ///
    /// # Errors
    ///
    /// Returns a storage failure.
    fn get_string(&self, key: &str) -> SaResult<Option<String>> {
        Ok(self.get(key)?.and_then(|value| match value {
            Value::Null => None,
            Value::String(text) => Some(text),
            other => Some(other.to_string()),
        }))
    }

    /// Reads an integer and returns zero for an absent or empty value.
    ///
    /// # Errors
    ///
    /// Returns a storage or conversion failure.
    fn get_i32(&self, key: &str) -> SaResult<i32>
    where
        Self: Sized,
    {
        self.get_with_default(key, 0)
    }

    /// Reads a long integer and returns zero for an absent or empty value.
    ///
    /// # Errors
    ///
    /// Returns a storage or conversion failure.
    fn get_i64(&self, key: &str) -> SaResult<i64>
    where
        Self: Sized,
    {
        self.get_with_default(key, 0)
    }

    /// Reads a double and returns zero for an absent or empty value.
    ///
    /// # Errors
    ///
    /// Returns a storage or conversion failure.
    fn get_f64(&self, key: &str) -> SaResult<f64>
    where
        Self: Sized,
    {
        self.get_with_default(key, 0.0)
    }

    /// Reads a float and returns zero for an absent or empty value.
    ///
    /// # Errors
    ///
    /// Returns a storage or conversion failure.
    fn get_f32(&self, key: &str) -> SaResult<f32>
    where
        Self: Sized,
    {
        self.get_with_default(key, 0.0)
    }

    /// Reads and deserializes a model.
    ///
    /// # Errors
    ///
    /// Returns a storage or conversion failure.
    fn get_model<T>(&self, key: &str) -> SaResult<Option<T>>
    where
        T: DeserializeOwned,
        Self: Sized,
    {
        self.get(key)?.map(convert_value).transpose()
    }

    /// Reads a model or returns the supplied default for null/empty values.
    ///
    /// # Errors
    ///
    /// Returns a storage or conversion failure.
    fn get_model_or_default<T>(&self, key: &str, default_value: T) -> SaResult<T>
    where
        T: DeserializeOwned,
        Self: Sized,
    {
        self.get_with_default(key, default_value)
    }

    /// Returns whether a non-null, non-empty value exists.
    ///
    /// # Errors
    ///
    /// Returns a storage failure.
    fn has(&self, key: &str) -> SaResult<bool> {
        let value = self.get(key)?;
        Ok(!Self::value_is_null(value.as_ref()))
    }
}

fn convert_value<T: DeserializeOwned>(value: Value) -> SaResult<T> {
    let fallback = match &value {
        Value::String(text) => serde_json::from_str(text),
        _ => serde_json::from_value(value.clone()),
    };
    serde_json::from_value(value)
        .or(fallback)
        .map_err(|error| SaTokenException::JsonConvert {
            message: error.to_string(),
        })
}
