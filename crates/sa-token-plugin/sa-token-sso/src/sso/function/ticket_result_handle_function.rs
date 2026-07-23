use crate::sso::exception::SaSsoException;
use crate::sso::model::SaCheckTicketResult;
use serde_json::Value;
use std::sync::Arc;

/// Handles a verified ticket and its final redirect target.
pub type TicketResultHandleFunction = Arc<
    dyn Fn(&SaCheckTicketResult, &str) -> Result<Value, SaSsoException> + Send + Sync + 'static,
>;
