//! Typed API Key errors.

pub mod api_key_exception;
pub mod api_key_scope_exception;

pub use api_key_exception::ApiKeyException;
pub use api_key_scope_exception::ApiKeyScopeException;
