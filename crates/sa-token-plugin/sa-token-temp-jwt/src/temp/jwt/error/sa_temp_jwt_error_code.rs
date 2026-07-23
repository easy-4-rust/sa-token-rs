//! Error codes for `sa-token-temp-jwt` (Java `SaTempJwtErrorCode`).

/// Java-compatible temp-jwt error codes.
pub struct SaTempJwtErrorCode;

impl SaTempJwtErrorCode {
    /// JWT mode secret key is missing.
    pub const CODE_30301: i32 = 30_301;
    /// JWT mode cannot delete tokens.
    pub const CODE_30302: i32 = 30_302;
    /// Token has expired.
    pub const CODE_30303: i32 = 30_303;
    /// JWT mode cannot list historical tokens.
    pub const CODE_30304: i32 = 30_304;
}
