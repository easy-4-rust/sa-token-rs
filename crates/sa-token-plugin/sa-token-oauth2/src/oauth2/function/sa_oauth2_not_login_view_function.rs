use serde_json::Value;

/// Renders the response shown when the authorization-server session is not logged in.
pub trait SaOAuth2NotLoginViewFunction: Send + Sync {
    fn get(&self) -> Value;
}

impl<F> SaOAuth2NotLoginViewFunction for F
where
    F: Fn() -> Value + Send + Sync,
{
    fn get(&self) -> Value {
        self()
    }
}
