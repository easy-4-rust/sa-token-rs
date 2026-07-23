use super::SaSsoClientInfo;
use serde::{Deserialize, Serialize};
/// Deprecated compatibility model wrapping [`SaSsoClientInfo`].
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct SaSsoClientModel {
    #[serde(flatten)]
    pub info: SaSsoClientInfo,
}
