use serde::{Deserialize, Serialize};
use std::fmt;

/// SSO client configuration with Java-compatible defaults.
#[derive(Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SaSsoClientConfig {
    pub mode: String,
    pub client: Option<String>,
    pub server_url: Option<String>,
    pub auth_url: String,
    pub signout_url: String,
    pub push_url: String,
    pub get_data_url: String,
    pub curr_sso_login: Option<String>,
    pub curr_sso_logout_call: Option<String>,
    pub is_http: bool,
    pub is_slo: bool,
    pub reg_logout_call: bool,
    pub secret_key: Option<String>,
    pub is_check_sign: bool,
}

impl fmt::Debug for SaSsoClientConfig {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("SaSsoClientConfig")
            .field("mode", &self.mode)
            .field("client", &self.client)
            .field("server_url", &self.server_url)
            .field("auth_url", &self.auth_url)
            .field("signout_url", &self.signout_url)
            .field("push_url", &self.push_url)
            .field("get_data_url", &self.get_data_url)
            .field("curr_sso_login", &self.curr_sso_login)
            .field("curr_sso_logout_call", &self.curr_sso_logout_call)
            .field("is_http", &self.is_http)
            .field("is_slo", &self.is_slo)
            .field("reg_logout_call", &self.reg_logout_call)
            .field(
                "secret_key",
                &self.secret_key.as_ref().map(|_| "[REDACTED]"),
            )
            .field("is_check_sign", &self.is_check_sign)
            .finish()
    }
}
impl Default for SaSsoClientConfig {
    fn default() -> Self {
        Self {
            mode: String::new(),
            client: None,
            server_url: None,
            auth_url: "/sso/auth".into(),
            signout_url: "/sso/signout".into(),
            push_url: "/sso/pushS".into(),
            get_data_url: "/sso/getData".into(),
            curr_sso_login: None,
            curr_sso_logout_call: None,
            is_http: false,
            is_slo: true,
            reg_logout_call: false,
            secret_key: None,
            is_check_sign: true,
        }
    }
}
impl SaSsoClientConfig {
    pub fn splicing_auth_url(&self) -> String {
        splice(self.server_url.as_deref(), &self.auth_url)
    }
    pub fn splicing_get_data_url(&self) -> String {
        splice(self.server_url.as_deref(), &self.get_data_url)
    }
    pub fn splicing_signout_url(&self) -> String {
        splice(self.server_url.as_deref(), &self.signout_url)
    }
    pub fn splicing_push_url(&self) -> String {
        splice(self.server_url.as_deref(), &self.push_url)
    }
}
fn splice(base: Option<&str>, path: &str) -> String {
    match base.filter(|value| !value.is_empty()) {
        Some(base) => format!(
            "{}/{}",
            base.trim_end_matches('/'),
            path.trim_start_matches('/')
        ),
        None => path.into(),
    }
}
