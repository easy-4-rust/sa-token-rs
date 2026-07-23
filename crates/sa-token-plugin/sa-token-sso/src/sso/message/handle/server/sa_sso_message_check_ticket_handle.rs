use crate::sso::exception::SaSsoException;
use crate::sso::function::SaSsoMessageHandleFunction;
use crate::sso::message::{SaSsoMessage, SaSsoMessageHandle};
use crate::sso::template::SaSsoTemplate;
use crate::sso::util::SaSsoConsts;
use serde_json::Value;

/// Handles a client request to validate and consume an SSO ticket.
pub struct SaSsoMessageCheckTicketHandle {
    callback: SaSsoMessageHandleFunction,
}

impl SaSsoMessageCheckTicketHandle {
    /// Creates the handler with runtime-specific ticket behavior.
    pub fn new(callback: SaSsoMessageHandleFunction) -> Self {
        Self { callback }
    }
}

impl SaSsoMessageHandle for SaSsoMessageCheckTicketHandle {
    fn handler_type(&self) -> &str {
        SaSsoConsts::MESSAGE_CHECK_TICKET
    }

    fn handle(
        &self,
        template: &SaSsoTemplate,
        message: &SaSsoMessage,
    ) -> Result<Value, SaSsoException> {
        (self.callback)(template, message)
    }
}
