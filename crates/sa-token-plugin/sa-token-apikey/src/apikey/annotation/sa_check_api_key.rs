//! Runtime metadata corresponding to Java `@SaCheckApiKey`.

use sa_token_core::annotation::sa_mode::SaMode;

/// Required scopes and AND/OR validation mode.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SaCheckApiKey {
    /// Required scopes.
    pub scopes: Vec<String>,
    /// Validation mode.
    pub mode: SaMode,
}
