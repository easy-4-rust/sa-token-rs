/// Explicit SSO protocol failure with a Java-compatible detailed code.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("SSO error[{code}]: {message}")]
pub struct SaSsoException {
    pub code: i32,
    pub message: String,
}
impl SaSsoException {
    pub fn new(code: i32, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }
}
