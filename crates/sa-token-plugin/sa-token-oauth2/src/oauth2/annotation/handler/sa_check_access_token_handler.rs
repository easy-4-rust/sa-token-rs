use crate::oauth2::annotation::SaCheckAccessToken;
use crate::oauth2::annotation::handler::{SaOAuth2AnnotationValidator, Validator};

/// Applies [`SaCheckAccessToken`] through an explicit runtime validator.
pub struct SaCheckAccessTokenHandler<V: SaOAuth2AnnotationValidator> {
    validator: Validator<V>,
}

impl<V: SaOAuth2AnnotationValidator> SaCheckAccessTokenHandler<V> {
    pub fn new(validator: Validator<V>) -> Self {
        Self { validator }
    }

    /// Validates the annotation scopes.
    ///
    /// # Errors
    ///
    /// Returns the runtime validator failure.
    pub fn check(&self, annotation: &SaCheckAccessToken) -> Result<(), V::Error> {
        self.validator.check_access_token_scope(&annotation.scope)
    }
}
