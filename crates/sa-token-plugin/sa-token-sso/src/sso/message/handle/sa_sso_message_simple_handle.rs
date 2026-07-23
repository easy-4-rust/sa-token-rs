use crate::sso::exception::SaSsoException;
use crate::sso::function::SaSsoMessageHandleFunction;
use crate::sso::message::SaSsoMessage;
use crate::sso::message::handle::SaSsoMessageHandle;
use crate::sso::template::SaSsoTemplate;
use serde_json::Value;

/// Message handler backed by a closure.
pub struct SaSsoMessageSimpleHandle {
    handler_type: String,
    callback: SaSsoMessageHandleFunction,
}

impl SaSsoMessageSimpleHandle {
    /// Creates a closure-backed handler.
    pub fn new(handler_type: impl Into<String>, callback: SaSsoMessageHandleFunction) -> Self {
        Self {
            handler_type: handler_type.into(),
            callback,
        }
    }
}

impl SaSsoMessageHandle for SaSsoMessageSimpleHandle {
    fn handler_type(&self) -> &str {
        &self.handler_type
    }

    fn handle(
        &self,
        template: &SaSsoTemplate,
        message: &SaSsoMessage,
    ) -> Result<Value, SaSsoException> {
        (self.callback)(template, message)
    }
}
