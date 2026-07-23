use crate::sso::config::SaSsoClientModel;
use crate::sso::exception::SaSsoException;
use crate::sso::model::TicketModel;
use crate::sso::template::SaSsoServerTemplate;
use serde_json::Value;

/// Instance-scoped facade for SSO server template operations.
pub struct SaSsoServerUtil<'a> {
    template: &'a SaSsoServerTemplate,
}

impl<'a> SaSsoServerUtil<'a> {
    /// Creates a facade over an explicit server template.
    pub fn new(template: &'a SaSsoServerTemplate) -> Self {
        Self { template }
    }

    /// Returns the underlying template.
    pub fn template(&self) -> &'a SaSsoServerTemplate {
        self.template
    }

    /// Creates and saves a ticket.
    ///
    /// # Errors
    ///
    /// Returns DAO or serialization failures.
    pub fn create_ticket_and_save(
        &self,
        client: impl Into<String>,
        login_id: Value,
        token_value: impl Into<String>,
    ) -> Result<String, SaSsoException> {
        self.template
            .create_ticket_and_save(client, login_id, token_value)
    }

    /// Loads a ticket.
    ///
    /// # Errors
    ///
    /// Returns DAO or deserialization failures.
    pub fn get_ticket(&self, ticket: &str) -> Result<Option<TicketModel>, SaSsoException> {
        self.template.get_ticket(ticket)
    }

    /// Returns a ticket's login ID.
    ///
    /// # Errors
    ///
    /// Returns DAO or deserialization failures.
    pub fn login_id(&self, ticket: &str) -> Result<Option<Value>, SaSsoException> {
        self.template.login_id(ticket)
    }

    /// Validates and consumes a ticket.
    ///
    /// # Errors
    ///
    /// Returns ticket/client validation or DAO failures.
    pub fn check_ticket_and_delete(
        &self,
        ticket: &str,
        client: &str,
    ) -> Result<TicketModel, SaSsoException> {
        self.template.check_ticket_and_delete(ticket, client)
    }

    /// Returns a configured client.
    ///
    /// # Errors
    ///
    /// Returns client validation failures.
    pub fn client(&self, client: &str) -> Result<SaSsoClientModel, SaSsoException> {
        self.template.client(client)
    }

    /// Validates a redirect URL.
    ///
    /// # Errors
    ///
    /// Returns redirect/client validation failures.
    pub fn check_redirect_url(&self, client: &str, redirect: &str) -> Result<(), SaSsoException> {
        self.template.check_redirect_url(client, redirect)
    }

    /// Builds a ticket-bearing redirect URL.
    ///
    /// # Errors
    ///
    /// Returns redirect, ticket, serialization, or DAO failures.
    pub fn build_redirect_url(
        &self,
        client: &str,
        redirect: &str,
        login_id: Value,
        token_value: &str,
    ) -> Result<String, SaSsoException> {
        self.template
            .build_redirect_url(client, redirect, login_id, token_value)
    }

    /// Invalidates an SSO account/device session.
    ///
    /// # Errors
    ///
    /// Returns the auth runtime failure.
    pub fn sso_logout(
        &self,
        login_id: &Value,
        device_id: Option<String>,
    ) -> Result<(), SaSsoException> {
        self.template.sso_logout(login_id, device_id)
    }
}
