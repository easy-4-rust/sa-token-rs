use serde_json::Value;

/// Renders the authorization confirmation view.
pub trait SaOAuth2ConfirmViewFunction: Send + Sync {
    fn apply(&self, client_id: &str, scopes: &[String]) -> Value;
}

impl<F> SaOAuth2ConfirmViewFunction for F
where
    F: Fn(&str, &[String]) -> Value + Send + Sync,
{
    fn apply(&self, client_id: &str, scopes: &[String]) -> Value {
        self(client_id, scopes)
    }
}
