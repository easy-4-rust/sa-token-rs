use crate::oauth2::annotation::SaCheckClientToken;
use crate::oauth2::annotation::handler::{SaOAuth2AnnotationValidator, Validator};

/// Applies [`SaCheckClientToken`] through an explicit runtime validator.
pub struct SaCheckClientTokenHandler<V: SaOAuth2AnnotationValidator> {
    validator: Validator<V>,
}

impl<V: SaOAuth2AnnotationValidator> SaCheckClientTokenHandler<V> {
    pub fn new(validator: Validator<V>) -> Self {
        Self { validator }
    }

    /// Validates the annotation scopes.
    ///
    /// # Errors
    ///
    /// Returns the runtime validator failure.
    pub fn check(&self, annotation: &SaCheckClientToken) -> Result<(), V::Error> {
        self.validator.check_client_token_scope(&annotation.scope)
    }
}
