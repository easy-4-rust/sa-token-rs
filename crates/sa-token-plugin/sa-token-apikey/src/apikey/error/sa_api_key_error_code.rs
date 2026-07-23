//! Error code registry corresponding to Java `SaApiKeyErrorCode`.

/// Namespace for stable API Key error codes.
pub struct SaApiKeyErrorCode;

impl SaApiKeyErrorCode {
    /// Invalid key.
    pub const CODE_12301: i32 = 12301;
    /// Expired key.
    pub const CODE_12302: i32 = 12302;
    /// Disabled key.
    pub const CODE_12303: i32 = 12303;
    /// Model validation failure.
    pub const CODE_12304: i32 = 12304;
    /// Index feature disabled.
    pub const CODE_12305: i32 = 12305;
    /// Missing scope.
    pub const CODE_12311: i32 = 12311;
    /// Login id mismatch.
    pub const CODE_12312: i32 = 12312;
}
