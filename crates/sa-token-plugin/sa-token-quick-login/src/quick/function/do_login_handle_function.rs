//! Login handler function trait (Java `DoLoginHandleFunction`).

use sa_token_core::util::sa_result::SaResultData;
use serde_json::Value;

/// `(name, pwd) -> login result` strategy hook.
pub trait DoLoginHandleFunction: Send + Sync {
    /// Validates credentials and performs login.
    fn apply(&self, name: &str, pwd: &str) -> SaResultData<Value>;
}

impl<F> DoLoginHandleFunction for F
where
    F: Fn(&str, &str) -> SaResultData<Value> + Send + Sync,
{
    fn apply(&self, name: &str, pwd: &str) -> SaResultData<Value> {
        self(name, pwd)
    }
}
