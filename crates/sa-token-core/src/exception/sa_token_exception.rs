//! Canonical Java `SaTokenException` mapping.
//!
//! Java models inheritance and a mutable numeric code. Rust represents the
//! complete exception hierarchy as the typed sum type in the parent module.

/// Rust counterpart of Java `SaTokenException` and its subclasses.
pub type SaTokenException = super::SaTokenException;

/// Java's undefined detailed error code.
pub const CODE_UNDEFINED: i32 = -1;

/// Returns an error when `condition` is true.
///
/// # Errors
/// Returns [`SaTokenException::Other`] with the supplied message.
pub fn not_true(condition: bool, message: impl Into<String>) -> super::SaResult<()> {
    not_true_with_code(condition, message, CODE_UNDEFINED)
}

/// Returns a coded error when `condition` is true.
///
/// # Errors
/// Returns a framework error carrying `code` and `message`.
pub fn not_true_with_code(
    condition: bool,
    message: impl Into<String>,
    code: i32,
) -> super::SaResult<()> {
    if condition {
        Err(SaTokenException::with_code(code, message))
    } else {
        Ok(())
    }
}

/// Returns an error when the optional value is absent.
///
/// # Errors
/// Returns [`SaTokenException::Other`] with the supplied message.
pub fn not_empty<T>(value: Option<T>, message: impl Into<String>) -> super::SaResult<T> {
    not_empty_with_code(value, message, CODE_UNDEFINED)
}

/// Returns a coded error when the optional value is absent.
///
/// # Errors
/// Returns a framework error carrying `code` and `message`.
pub fn not_empty_with_code<T>(
    value: Option<T>,
    message: impl Into<String>,
    code: i32,
) -> super::SaResult<T> {
    value.ok_or_else(|| SaTokenException::with_code(code, message))
}

#[cfg(test)]
mod tests {
    use super::{not_empty, not_empty_with_code, not_true, not_true_with_code};

    #[test]
    fn assertions_return_typed_errors() {
        assert!(not_true(true, "boom").is_err());
        assert!(not_true(false, "unused").is_ok());
        assert_eq!(not_empty(Some(7), "missing").expect("present"), 7);
        assert!(not_empty::<i32>(None, "missing").is_err());
        let coded = not_true_with_code(true, "coded", 11001).expect_err("must fail");
        assert_eq!(coded.code(), 11001);
        let coded = not_empty_with_code::<i32>(None, "missing", 11002).expect_err("must fail");
        assert_eq!(coded.code(), 11002);
    }
}
