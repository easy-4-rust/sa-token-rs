use crate::sso::error::SaSsoErrorCode;
use crate::sso::exception::SaSsoException;
use crate::sso::message::SaSsoMessage;
use crate::sso::message::handle::SaSsoMessageHandle;
use crate::sso::template::SaSsoTemplate;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// Thread-safe registry of SSO message handlers.
#[derive(Default)]
pub struct SaSsoMessageHolder {
    handlers: RwLock<HashMap<String, Arc<dyn SaSsoMessageHandle>>>,
}

impl SaSsoMessageHolder {
    /// Creates an empty handler registry.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns whether a handler exists for `message_type`.
    pub fn has_handle(&self, message_type: &str) -> bool {
        self.handlers
            .read()
            .map(|handlers| handlers.contains_key(message_type))
            .unwrap_or(false)
    }

    /// Registers or replaces a handler.
    ///
    /// # Errors
    ///
    /// Returns an error if the registry lock was poisoned.
    pub fn add_handle(
        &self,
        handler: Arc<dyn SaSsoMessageHandle>,
    ) -> Result<Option<Arc<dyn SaSsoMessageHandle>>, SaSsoException> {
        let message_type = handler.handler_type().to_owned();
        Ok(self
            .handlers
            .write()
            .map_err(lock_error)?
            .insert(message_type, handler))
    }

    /// Removes a handler.
    ///
    /// # Errors
    ///
    /// Returns an error if the registry lock was poisoned.
    pub fn remove_handle(
        &self,
        message_type: &str,
    ) -> Result<Option<Arc<dyn SaSsoMessageHandle>>, SaSsoException> {
        Ok(self
            .handlers
            .write()
            .map_err(lock_error)?
            .remove(message_type))
    }

    /// Returns a cloned handler reference.
    ///
    /// # Errors
    ///
    /// Returns an error if the registry lock was poisoned.
    pub fn get_handle(
        &self,
        message_type: &str,
    ) -> Result<Option<Arc<dyn SaSsoMessageHandle>>, SaSsoException> {
        Ok(self
            .handlers
            .read()
            .map_err(lock_error)?
            .get(message_type)
            .cloned())
    }

    /// Dispatches a message to its registered handler.
    ///
    /// # Errors
    ///
    /// Returns code `30021` when no matching handler exists, or forwards the
    /// handler failure.
    pub fn handle_message(
        &self,
        template: &SaSsoTemplate,
        message: &SaSsoMessage,
    ) -> Result<Value, SaSsoException> {
        let message_type = message.check_type()?;
        let handler = self.get_handle(message_type)?.ok_or_else(|| {
            SaSsoException::new(
                SaSsoErrorCode::CODE_30021,
                format!("SSO message handler not found: {message_type}"),
            )
        })?;
        handler.handle(template, message)
    }
}

fn lock_error<T>(_: std::sync::PoisonError<T>) -> SaSsoException {
    SaSsoException::new(
        SaSsoErrorCode::CODE_30021,
        "SSO message handler registry is unavailable",
    )
}
