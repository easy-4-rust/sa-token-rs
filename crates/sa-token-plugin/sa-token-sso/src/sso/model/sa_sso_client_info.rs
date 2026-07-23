use crate::sso::util::SaSsoConsts;
use serde::{Deserialize, Serialize};
/// Client login registration stored in an account session.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct SaSsoClientInfo {
    pub mode: i32,
    pub client: String,
    pub slo_callback_url: String,
    pub reg_time: i64,
    pub index: i32,
}
impl SaSsoClientInfo {
    pub fn mode_three(client: impl Into<String>, callback: impl Into<String>, index: i32) -> Self {
        Self {
            mode: SaSsoConsts::SSO_MODE_3,
            client: client.into(),
            slo_callback_url: callback.into(),
            reg_time: chrono::Utc::now().timestamp_millis(),
            index,
        }
    }
}
