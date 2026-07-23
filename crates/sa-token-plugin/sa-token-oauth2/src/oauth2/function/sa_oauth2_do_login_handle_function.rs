use serde_json::Value;

/// Handles a login request submitted to the OAuth2 authorization server.
pub trait SaOAuth2DoLoginHandleFunction: Send + Sync {
    fn apply(&self, name: &str, password: &str) -> Value;
}

impl<F> SaOAuth2DoLoginHandleFunction for F
where
    F: Fn(&str, &str) -> Value + Send + Sync,
{
    fn apply(&self, name: &str, password: &str) -> Value {
        self(name, password)
    }
}
