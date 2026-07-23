use std::collections::BTreeMap;
use std::fmt;

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Claims used to build an OpenID Connect ID token.
#[derive(Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct IdTokenModel {
    pub iss: Option<String>,
    pub sub: Option<Value>,
    pub aud: Option<String>,
    pub exp: i64,
    pub iat: i64,
    pub auth_time: i64,
    pub nonce: Option<String>,
    pub acr: Option<String>,
    pub amr: Option<String>,
    pub azp: Option<String>,
    pub extra_data: Option<BTreeMap<String, Value>>,
}

impl fmt::Debug for IdTokenModel {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("IdTokenModel")
            .field("iss", &self.iss)
            .field("sub", &self.sub)
            .field("aud", &self.aud)
            .field("exp", &self.exp)
            .field("iat", &self.iat)
            .field("auth_time", &self.auth_time)
            .field("nonce", &self.nonce.as_ref().map(|_| "***"))
            .field("acr", &self.acr)
            .field("amr", &self.amr)
            .field("azp", &self.azp)
            .field("extra_data", &self.extra_data)
            .finish()
    }
}
