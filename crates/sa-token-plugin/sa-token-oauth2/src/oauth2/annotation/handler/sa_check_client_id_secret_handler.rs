use crate::oauth2::annotation::SaCheckClientIdSecret;
use crate::oauth2::annotation::handler::{SaOAuth2AnnotationValidator, Validator};

/// Applies [`SaCheckClientIdSecret`] through an explicit runtime validator.
pub struct SaCheckClientIdSecretHandler<V: SaOAuth2AnnotationValidator> {
    validator: Validator<V>,
}

impl<V: SaOAuth2AnnotationValidator> SaCheckClientIdSecretHandler<V> {
    pub fn new(validator: Validator<V>) -> Self {
        Self { validator }
    }

    /// Validates client credentials.
    ///
    /// # Errors
    ///
    /// Returns the runtime validator failure.
    pub fn check(&self, _: &SaCheckClientIdSecret) -> Result<(), V::Error> {
        self.validator.check_client_id_secret()
    }
}
