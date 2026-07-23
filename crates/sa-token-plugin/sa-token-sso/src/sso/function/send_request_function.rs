use crate::sso::exception::SaSsoException;
use std::sync::Arc;

/// Sends an SSO protocol request and returns its response body.
///
/// Transport failures remain explicit instead of being converted to an empty
/// response.
pub type SendRequestFunction =
    Arc<dyn Fn(&str) -> Result<String, SaSsoException> + Send + Sync + 'static>;
