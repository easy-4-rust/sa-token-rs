use crate::sso::exception::SaSsoException;
use crate::sso::message::SaSsoMessage;
use crate::sso::template::SaSsoClientTemplate;
use serde_json::Value;
use std::collections::HashMap;

/// Instance-scoped facade for SSO client template operations.
pub struct SaSsoClientUtil<'a> {
    template: &'a SaSsoClientTemplate,
}

impl<'a> SaSsoClientUtil<'a> {
    /// Creates a facade over an explicit client template.
    pub fn new(template: &'a SaSsoClientTemplate) -> Self {
        Self { template }
    }

    /// Returns the underlying template.
    pub fn template(&self) -> &'a SaSsoClientTemplate {
        self.template
    }

    /// Sends a signed custom data request.
    ///
    /// # Errors
    ///
    /// Returns URL/signing or transport failures.
    pub fn get_data(
        &self,
        path: &str,
        params: &HashMap<String, Value>,
    ) -> Result<String, SaSsoException> {
        self.template.get_data(path, params)
    }

    /// Builds the SSO center authorization URL.
    ///
    /// # Errors
    ///
    /// Returns invalid URL failures.
    pub fn build_server_auth_url(
        &self,
        client_login_url: &str,
        back: Option<&str>,
    ) -> Result<String, SaSsoException> {
        self.template.build_server_auth_url(client_login_url, back)
    }

    /// Pushes a signed message.
    ///
    /// # Errors
    ///
    /// Returns message, URL, signing, or transport failures.
    pub fn push_message(&self, message: &mut SaSsoMessage) -> Result<String, SaSsoException> {
        self.template.push_message(message)
    }

    /// Builds a check-ticket message.
    pub fn build_check_ticket_message(
        &self,
        ticket: impl Into<String>,
        callback: Option<&str>,
    ) -> SaSsoMessage {
        self.template.build_check_ticket_message(ticket, callback)
    }

    /// Builds a sign-out message.
    pub fn build_signout_message(&self, login_id: Value, device_id: Option<&str>) -> SaSsoMessage {
        self.template.build_signout_message(login_id, device_id)
    }
}
