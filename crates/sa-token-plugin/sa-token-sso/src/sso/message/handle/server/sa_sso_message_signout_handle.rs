use crate::sso::exception::SaSsoException;
use crate::sso::function::SaSsoMessageHandleFunction;
use crate::sso::message::{SaSsoMessage, SaSsoMessageHandle};
use crate::sso::template::SaSsoTemplate;
use crate::sso::util::SaSsoConsts;
use serde_json::Value;

/// Handles an SSO client request to sign out from the center.
pub struct SaSsoMessageSignoutHandle {
    callback: SaSsoMessageHandleFunction,
}

impl SaSsoMessageSignoutHandle {
    /// Creates the handler with runtime-specific sign-out behavior.
    pub fn new(callback: SaSsoMessageHandleFunction) -> Self {
        Self { callback }
    }
}

impl SaSsoMessageHandle for SaSsoMessageSignoutHandle {
    fn handler_type(&self) -> &str {
        SaSsoConsts::MESSAGE_SIGNOUT
    }

    fn handle(
        &self,
        template: &SaSsoTemplate,
        message: &SaSsoMessage,
    ) -> Result<Value, SaSsoException> {
        (self.callback)(template, message)
    }
}
