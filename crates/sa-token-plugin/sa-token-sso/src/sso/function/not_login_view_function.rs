use serde_json::Value;
use std::sync::Arc;

/// Produces the response shown when the SSO center is not logged in.
pub type NotLoginViewFunction = Arc<dyn Fn() -> Value + Send + Sync + 'static>;
