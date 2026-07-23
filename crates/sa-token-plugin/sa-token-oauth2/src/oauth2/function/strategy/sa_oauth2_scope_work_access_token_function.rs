use crate::oauth2::data::model::AccessTokenModel;

/// Scope hook that enriches an access-token model.
pub trait SaOAuth2ScopeWorkAccessTokenFunction: Send + Sync {
    fn accept(&self, access_token: &mut AccessTokenModel);
}

impl<F> SaOAuth2ScopeWorkAccessTokenFunction for F
where
    F: Fn(&mut AccessTokenModel) + Send + Sync,
{
    fn accept(&self, access_token: &mut AccessTokenModel) {
        self(access_token);
    }
}
