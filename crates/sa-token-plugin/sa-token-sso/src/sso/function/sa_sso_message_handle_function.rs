use crate::sso::exception::SaSsoException;
use crate::sso::message::SaSsoMessage;
use crate::sso::template::SaSsoTemplate;
use serde_json::Value;
use std::sync::Arc;

/// Callback used by a simple SSO message handler.
pub type SaSsoMessageHandleFunction = Arc<
    dyn Fn(&SaSsoTemplate, &SaSsoMessage) -> Result<Value, SaSsoException> + Send + Sync + 'static,
>;
