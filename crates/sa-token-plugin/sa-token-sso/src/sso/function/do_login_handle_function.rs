use serde_json::Value;
use std::sync::Arc;

/// Handles the server-side username/password login request.
pub type DoLoginHandleFunction = Arc<dyn Fn(&str, &str) -> Value + Send + Sync + 'static>;
