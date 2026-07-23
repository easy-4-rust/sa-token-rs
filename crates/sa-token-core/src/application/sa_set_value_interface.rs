//! Common writes for application, session, and request storage.

use serde_json::Value;

use super::sa_get_value_interface::SaGetValueInterface;
use crate::exception::SaResult;

/// Common value-writing contract.
pub trait SaSetValueInterface: SaGetValueInterface {
    /// Stores a value without expiry.
    ///
    /// # Errors
    ///
    /// Returns a storage failure.
    fn set(&self, key: &str, value: Value) -> SaResult<&Self>;

    /// Stores a value with a TTL in seconds.
    ///
    /// # Errors
    ///
    /// Returns a storage failure.
    fn set_with_ttl(&self, key: &str, value: Value, ttl: i64) -> SaResult<&Self>;

    /// Deletes a value.
    ///
    /// # Errors
    ///
    /// Returns a storage failure.
    fn delete(&self, key: &str) -> SaResult<&Self>;

    /// Computes and stores a value only when the current value is absent.
    ///
    /// # Errors
    ///
    /// Returns a storage failure.
    fn get_or_insert_with<F>(&self, key: &str, supplier: F) -> SaResult<Value>
    where
        F: FnOnce() -> Value,
        Self: Sized,
    {
        if let Some(value) = self.get(key)? {
            if !Self::value_is_null(Some(&value)) {
                return Ok(value);
            }
        }
        let value = supplier();
        self.set(key, value.clone())?;
        Ok(value)
    }

    /// Stores a value only when the current value is absent or empty.
    ///
    /// # Errors
    ///
    /// Returns a storage failure.
    fn set_by_null(&self, key: &str, value: Value) -> SaResult<&Self>
    where
        Self: Sized,
    {
        if !self.has(key)? {
            self.set(key, value)?;
        }
        Ok(self)
    }
}
