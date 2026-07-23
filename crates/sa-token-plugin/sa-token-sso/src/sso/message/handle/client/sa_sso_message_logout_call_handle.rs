use crate::sso::exception::SaSsoException;
use crate::sso::function::SaSsoMessageHandleFunction;
use crate::sso::message::{SaSsoMessage, SaSsoMessageHandle};
use crate::sso::template::SaSsoTemplate;
use crate::sso::util::SaSsoConsts;
use serde_json::Value;

/// Handles a server-initiated logout callback on an SSO client.
pub struct SaSsoMessageLogoutCallHandle {
    callback: SaSsoMessageHandleFunction,
}

impl SaSsoMessageLogoutCallHandle {
    /// Creates the handler with runtime-specific logout behavior.
    pub fn new(callback: SaSsoMessageHandleFunction) -> Self {
        Self { callback }
    }
}

impl SaSsoMessageHandle for SaSsoMessageLogoutCallHandle {
    fn handler_type(&self) -> &str {
        SaSsoConsts::MESSAGE_LOGOUT_CALL
    }

    fn handle(
        &self,
        template: &SaSsoTemplate,
        message: &SaSsoMessage,
    ) -> Result<Value, SaSsoException> {
        (self.callback)(template, message)
    }
}
