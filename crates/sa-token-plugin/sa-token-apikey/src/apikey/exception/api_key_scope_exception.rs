//! API Key scope exception.

/// Error describing a missing scope.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("API Key scope error[{code}]: key={api_key}, scope={scope}")]
pub struct ApiKeyScopeException {
    /// Stable detailed code.
    pub code: i32,
    /// Offending API Key.
    pub api_key: String,
    /// Required scope.
    pub scope: String,
}
