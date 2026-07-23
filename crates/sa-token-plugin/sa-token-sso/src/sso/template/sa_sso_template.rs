use crate::sso::exception::SaSsoException;
use crate::sso::message::{SaSsoMessage, SaSsoMessageHolder};
use crate::sso::name::{ApiName, ParamName};
use sa_token_core::stp::stp_logic::StpLogic;
use serde_json::Value;
use std::sync::Arc;

/// Common SSO template state shared by client and server roles.
pub struct SaSsoTemplate {
    pub api_name: ApiName,
    pub param_name: ParamName,
    stp_logic: Option<Arc<StpLogic>>,
    pub message_holder: Arc<SaSsoMessageHolder>,
}

impl Default for SaSsoTemplate {
    fn default() -> Self {
        Self {
            api_name: ApiName::default(),
            param_name: ParamName::default(),
            stp_logic: None,
            message_holder: Arc::new(SaSsoMessageHolder::new()),
        }
    }
}

impl SaSsoTemplate {
    /// Creates a template using Java-compatible names.
    pub fn new() -> Self {
        Self::default()
    }

    /// Replaces the protocol parameter names.
    pub fn with_param_name(mut self, param_name: ParamName) -> Self {
        self.param_name = param_name;
        self
    }

    /// Replaces the protocol endpoint names.
    pub fn with_api_name(mut self, api_name: ApiName) -> Self {
        self.api_name = api_name;
        self
    }

    /// Uses an explicit authentication logic instance.
    pub fn with_stp_logic(mut self, stp_logic: Arc<StpLogic>) -> Self {
        self.stp_logic = Some(stp_logic);
        self
    }

    /// Returns the explicitly configured authentication logic.
    pub fn stp_logic(&self) -> Option<&Arc<StpLogic>> {
        self.stp_logic.as_ref()
    }

    /// Returns the configured logic or a Java-compatible default login logic.
    pub fn stp_logic_or_global(&self) -> Arc<StpLogic> {
        self.stp_logic
            .clone()
            .unwrap_or_else(|| Arc::new(StpLogic::new("login")))
    }

    /// Dispatches an SSO message through the local registry.
    ///
    /// # Errors
    ///
    /// Returns a protocol error when the message is invalid, no handler is
    /// registered, or the handler fails.
    pub fn handle_message(&self, message: &SaSsoMessage) -> Result<Value, SaSsoException> {
        self.message_holder.handle_message(self, message)
    }
}
