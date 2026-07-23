use crate::sso::error::SaSsoErrorCode;
use crate::sso::exception::SaSsoException;
use crate::sso::function::{
    CheckTicketAppendDataFunction, DoLoginHandleFunction, NotLoginViewFunction, SendRequestFunction,
};
use serde_json::{Value, json};
use std::sync::Arc;

/// Task accepted by the server's asynchronous execution policy.
pub type SaSsoTask = Box<dyn FnOnce() + Send + 'static>;

/// Executes a protocol side effect outside the current request.
pub type AsyncRunFunction = Arc<dyn Fn(SaSsoTask) + Send + Sync + 'static>;

/// Receives the final authorization redirect before it is returned.
pub type RedirectNoticeFunction = Arc<dyn Fn(&str) + Send + Sync + 'static>;

/// Customizable SSO server policies.
pub struct SaSsoServerStrategy {
    pub send_request: SendRequestFunction,
    pub async_run: AsyncRunFunction,
    pub not_login_view: NotLoginViewFunction,
    pub do_login_handle: DoLoginHandleFunction,
    pub jump_to_redirect_url_notice: RedirectNoticeFunction,
    pub check_ticket_append_data: CheckTicketAppendDataFunction,
}

impl Default for SaSsoServerStrategy {
    fn default() -> Self {
        Self {
            send_request: Arc::new(|_| {
                Err(SaSsoException::new(
                    SaSsoErrorCode::CODE_30001,
                    "SSO server HTTP transport is not configured",
                ))
            }),
            async_run: Arc::new(|task| {
                std::thread::spawn(task);
            }),
            not_login_view: Arc::new(|| {
                Value::String("当前会话在 SSO-Server 认证中心尚未登录（当前未配置登录视图）".into())
            }),
            do_login_handle: Arc::new(|_, _| json!({"code": 500, "msg": "error"})),
            jump_to_redirect_url_notice: Arc::new(|_| {}),
            check_ticket_append_data: Arc::new(|_, result| result),
        }
    }
}

impl SaSsoServerStrategy {
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
                format!("invalid SSO server response: {error}"),
            )
        })
    }
}
