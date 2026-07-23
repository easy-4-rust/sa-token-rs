/// Explicit API signature failure.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("signature error[{code}]: {message}")]
pub struct SaSignException {
    pub code: i32,
    pub message: String,
}
impl SaSignException {
    pub fn new(code: i32, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }
}
