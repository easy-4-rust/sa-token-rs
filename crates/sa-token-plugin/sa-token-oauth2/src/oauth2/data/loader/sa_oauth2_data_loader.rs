use crate::oauth2::data::model::loader::SaClientModel;
use crate::oauth2::exception::SaOAuth2ClientModelException;

/// Application/client data required by OAuth2 and OIDC handlers.
pub trait SaOAuth2DataLoader: Send + Sync {
    fn get_client_model(&self, client_id: &str) -> Option<SaClientModel>;
    fn get_openid(&self, client_id: &str, login_id: &str) -> String;
    fn get_unionid(&self, subject_id: &str, login_id: &str) -> String;
    fn get_higher_scope_list(&self) -> Vec<String>;
    fn get_lower_scope_list(&self) -> Vec<String>;

    /// Returns a client or Java-compatible code 30105.
    ///
    /// # Errors
    ///
    /// Returns a client-model error when `client_id` is unknown.
    fn get_client_model_not_null(
        &self,
        client_id: &str,
    ) -> Result<SaClientModel, SaOAuth2ClientModelException> {
        self.get_client_model(client_id).ok_or_else(|| {
            SaOAuth2ClientModelException::new(format!("无效 client_id: {client_id}"), 30105)
                .with_client_id(client_id)
        })
    }
}
