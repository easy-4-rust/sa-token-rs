use serde::{Deserialize, Serialize};
use serde_json::Value;
/// One-time SSO ticket record.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TicketModel {
    pub ticket: String,
    pub client: String,
    pub login_id: Value,
    pub token_value: String,
    pub create_time: i64,
}
impl TicketModel {
    pub fn new(
        ticket: impl Into<String>,
        client: impl Into<String>,
        login_id: Value,
        token_value: impl Into<String>,
    ) -> Self {
        Self {
            ticket: ticket.into(),
            client: client.into(),
            login_id,
            token_value: token_value.into(),
            create_time: chrono::Utc::now().timestamp_millis(),
        }
    }
}
