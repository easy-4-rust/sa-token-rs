use serde_json::Value;
use std::sync::Arc;

/// Appends application data to a successful check-ticket response.
pub type CheckTicketAppendDataFunction = Arc<dyn Fn(Value, Value) -> Value + Send + Sync + 'static>;
