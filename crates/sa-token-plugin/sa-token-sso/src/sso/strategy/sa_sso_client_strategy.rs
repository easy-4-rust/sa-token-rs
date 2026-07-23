use crate::sso::error::SaSsoErrorCode;
use crate::sso::exception::SaSsoException;
use crate::sso::function::{SendRequestFunction, TicketResultHandleFunction};
use serde_json::Value;
use std::sync::Arc;

/// Converts an identifier between the SSO center and a local application.
pub type IdConversionFunction = Arc<dyn Fn(Value) -> Value + Send + Sync + 'static>;

/// Customizable SSO client policies.
pub struct SaSsoClientStrategy {
    pub send_request: SendRequestFunction,
    pub ticket_result_handle: Option<TicketResultHandleFunction>,
    pub convert_center_id_to_login_id: IdConversionFunction,
    pub convert_login_id_to_center_id: IdConversionFunction,
}

impl Default for SaSsoClientStrategy {
    fn default() -> Self {
        Self {
            send_request: Arc::new(|_| {
                Err(SaSsoException::new(
                    SaSsoErrorCode::CODE_30001,
                    "SSO client HTTP transport is not configured",
                ))
            }),
            ticket_result_handle: None,
            convert_center_id_to_login_id: Arc::new(|center_id| center_id),
            convert_login_id_to_center_id: Arc::new(|login_id| login_id),
        }
    }
}

impl SaSsoClientStrategy {
    /// Sends a request and decodes the JSON response.
    ///
    /// # Errors
    ///
    /// Returns the transport error or code `30001` when the response is not
    /// valid JSON.
    pub fn request_as_result(&self, url: &str) -> Result<Value, SaSsoException> {
        let body = (self.send_request)(url)?;
        serde_json::from_str(&body).map_err(|error| {
            SaSsoException::new(
                SaSsoErrorCode::CODE_30001,
                format!("invalid SSO client response: {error}"),
            )
        })
    }
}
