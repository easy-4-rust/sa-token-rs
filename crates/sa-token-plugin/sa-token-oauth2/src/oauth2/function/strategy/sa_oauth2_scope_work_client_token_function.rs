use crate::oauth2::data::model::ClientTokenModel;

/// Scope hook that enriches a client-token model.
pub trait SaOAuth2ScopeWorkClientTokenFunction: Send + Sync {
    fn accept(&self, client_token: &mut ClientTokenModel);
}

impl<F> SaOAuth2ScopeWorkClientTokenFunction for F
where
    F: Fn(&mut ClientTokenModel) + Send + Sync,
{
    fn accept(&self, client_token: &mut ClientTokenModel) {
        self(client_token);
    }
}
