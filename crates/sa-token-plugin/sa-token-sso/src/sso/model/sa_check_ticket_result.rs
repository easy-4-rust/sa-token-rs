use serde::{Deserialize, Serialize};
use serde_json::Value;
/// Result returned after a successful ticket check.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct SaCheckTicketResult {
    pub login_id: Option<Value>,
    pub token_value: Option<String>,
    pub device_id: Option<String>,
    pub remain_token_timeout: Option<i64>,
    pub remain_session_timeout: Option<i64>,
    pub center_id: Option<Value>,
    pub result: Option<Value>,
}
