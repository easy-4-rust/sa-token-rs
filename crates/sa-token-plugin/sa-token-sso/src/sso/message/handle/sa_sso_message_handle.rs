use crate::sso::exception::SaSsoException;
use crate::sso::message::SaSsoMessage;
use crate::sso::template::SaSsoTemplate;
use serde_json::Value;

/// Object-safe SSO message handler contract.
pub trait SaSsoMessageHandle: Send + Sync {
    /// Returns the message type accepted by this handler.
    fn handler_type(&self) -> &str;

    /// Handles one protocol message.
    ///
    /// # Errors
    ///
    /// Returns a protocol error when validation or processing fails.
    fn handle(
        &self,
        template: &SaSsoTemplate,
        message: &SaSsoMessage,
    ) -> Result<Value, SaSsoException>;
}
