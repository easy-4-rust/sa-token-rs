use super::SaSsoClientModel;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

/// SSO server configuration with Java-compatible defaults.
#[derive(Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SaSsoServerConfig {
    pub mode: String,
    pub ticket_timeout: i64,
    pub home_route: Option<String>,
    pub is_slo: bool,
    pub auto_renew_timeout: bool,
    pub max_reg_client: i32,
    pub is_check_sign: bool,
    pub clients: HashMap<String, SaSsoClientModel>,
    pub allow_anon_client: bool,
    pub allow_url: String,
    pub secret_key: Option<String>,
}

impl fmt::Debug for SaSsoServerConfig {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("SaSsoServerConfig")
            .field("mode", &self.mode)
            .field("ticket_timeout", &self.ticket_timeout)
            .field("home_route", &self.home_route)
            .field("is_slo", &self.is_slo)
            .field("auto_renew_timeout", &self.auto_renew_timeout)
            .field("max_reg_client", &self.max_reg_client)
            .field("is_check_sign", &self.is_check_sign)
            .field("clients", &self.clients)
            .field("allow_anon_client", &self.allow_anon_client)
            .field("allow_url", &self.allow_url)
            .field(
                "secret_key",
                &self.secret_key.as_ref().map(|_| "[REDACTED]"),
            )
            .finish()
    }
}
impl Default for SaSsoServerConfig {
    fn default() -> Self {
        Self {
            mode: String::new(),
            ticket_timeout: 300,
            home_route: None,
            is_slo: true,
            auto_renew_timeout: false,
            max_reg_client: 32,
            is_check_sign: true,
            clients: HashMap::new(),
            allow_anon_client: false,
            allow_url: String::new(),
            secret_key: None,
        }
    }
}
impl SaSsoServerConfig {
    pub fn add_client(&mut self, client: SaSsoClientModel) {
        self.clients.insert(client.client.clone(), client);
    }
}
