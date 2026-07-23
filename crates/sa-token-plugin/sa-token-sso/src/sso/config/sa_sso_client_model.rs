use crate::sso::error::SaSsoErrorCode;
use crate::sso::exception::SaSsoException;
use serde::{Deserialize, Serialize};
use std::fmt;
use url::Url;

/// Client registration accepted by the SSO server.
#[derive(Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SaSsoClientModel {
    pub client: String,
    pub allow_url: String,
    pub is_push: bool,
    pub is_slo: bool,
    pub secret_key: Option<String>,
    pub server_url: Option<String>,
    pub push_url: String,
}

impl fmt::Debug for SaSsoClientModel {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("SaSsoClientModel")
            .field("client", &self.client)
            .field("allow_url", &self.allow_url)
            .field("is_push", &self.is_push)
            .field("is_slo", &self.is_slo)
            .field(
                "secret_key",
                &self.secret_key.as_ref().map(|_| "[REDACTED]"),
            )
            .field("server_url", &self.server_url)
            .field("push_url", &self.push_url)
            .finish()
    }
}
impl Default for SaSsoClientModel {
    fn default() -> Self {
        Self {
            client: String::new(),
            allow_url: String::new(),
            is_push: false,
            is_slo: true,
            secret_key: None,
            server_url: None,
            push_url: "/sso/pushC".into(),
        }
    }
}
impl SaSsoClientModel {
    pub fn splicing_push_url(&self) -> Result<String, SaSsoException> {
        let value = match self.server_url.as_deref().filter(|value| !value.is_empty()) {
            Some(base) => format!(
                "{}/{}",
                base.trim_end_matches('/'),
                self.push_url.trim_start_matches('/')
            ),
            None => self.push_url.clone(),
        };
        Url::parse(&value).map_err(|_| {
            SaSsoException::new(SaSsoErrorCode::CODE_30023, "invalid client push URL")
        })?;
        Ok(value)
    }
}
