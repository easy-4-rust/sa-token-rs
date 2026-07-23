use crate::sso::error::SaSsoErrorCode;
use crate::sso::exception::SaSsoException;
use crate::sso::message::SaSsoMessage;
use crate::sso::model::SaCheckTicketResult;
use crate::sso::processor::{SaSsoProcessorHelper, SaSsoProcessorResult, SaSsoRequest};
use crate::sso::strategy::SaSsoClientStrategy;
use crate::sso::template::{SaSsoClientTemplate, SaSsoServerTemplate};
use crate::sso::util::SaSsoConsts;
use serde_json::{Map, Value};
use std::collections::HashMap;
use std::sync::Arc;

/// Local-session operations required by the SSO client processor.
pub trait SaSsoClientSession: Send + Sync + 'static {
    /// Returns whether the local request is authenticated.
    fn is_login(&self) -> Result<bool, SaSsoException>;

    /// Creates the local session from a checked ticket.
    fn login(&self, result: &SaCheckTicketResult) -> Result<(), SaSsoException>;

    /// Returns the local login ID for sign-out.
    fn login_id(&self) -> Result<Option<Value>, SaSsoException>;

    /// Returns the local login device ID.
    fn device_id(&self) -> Result<Option<String>, SaSsoException>;

    /// Invalidates the current local session.
    fn logout(&self) -> Result<(), SaSsoException>;
}

/// Framework-neutral SSO client route processor.
pub struct SaSsoClientProcessor {
    pub template: Arc<SaSsoClientTemplate>,
    session: Arc<dyn SaSsoClientSession>,
    direct_server: Option<Arc<SaSsoServerTemplate>>,
}

impl SaSsoClientProcessor {
    /// Creates a client processor.
    pub fn new(template: Arc<SaSsoClientTemplate>, session: Arc<dyn SaSsoClientSession>) -> Self {
        Self {
            template,
            session,
            direct_server: None,
        }
    }

    /// Enables mode-two direct ticket validation against an explicit server.
    pub fn with_direct_server(mut self, server: Arc<SaSsoServerTemplate>) -> Self {
        self.direct_server = Some(server);
        self
    }

    /// Dispatches all SSO client routes.
    ///
    /// # Errors
    ///
    /// Returns signing, ticket, transport, session, or protocol failures.
    pub fn dispatch(&self, request: &SaSsoRequest) -> Result<SaSsoProcessorResult, SaSsoException> {
        let api = &self.template.common.api_name;
        if request.path == api.sso_login {
            self.sso_login(request)
        } else if request.path == api.sso_logout {
            self.sso_logout(request)
        } else if request.path == api.sso_push_c {
            self.sso_push(request)
        } else if request.path == api.sso_logout_call && self.template.config().reg_logout_call {
            self.sso_logout_call(request)
        } else {
            Ok(SaSsoProcessorResult::NotHandled)
        }
    }

    fn sso_login(&self, request: &SaSsoRequest) -> Result<SaSsoProcessorResult, SaSsoException> {
        let params = &self.template.common.param_name;
        let back = request.param(&params.back).unwrap_or("/");
        let Some(ticket) = request.param(&params.ticket) else {
            if self.session.is_login()? {
                return Ok(SaSsoProcessorResult::Redirect(back.to_owned()));
            }
            let current_url = self
                .template
                .config()
                .curr_sso_login
                .as_deref()
                .unwrap_or(&request.path);
            return self
                .template
                .build_server_auth_url(current_url, Some(back))
                .map(SaSsoProcessorResult::Redirect);
        };

        let checked = self.check_ticket(ticket, Some(&request.path))?;
        if let Some(handler) = &self.template.strategy.ticket_result_handle {
            return handler(&checked, back).map(SaSsoProcessorResult::Json);
        }
        self.session.login(&checked)?;
        Ok(SaSsoProcessorResult::Redirect(back.to_owned()))
    }

    fn sso_logout(&self, request: &SaSsoRequest) -> Result<SaSsoProcessorResult, SaSsoException> {
        if !self.template.config().is_slo {
            return Ok(SaSsoProcessorResult::NotHandled);
        }
        if let Some(login_id) = self.session.login_id()? {
            let center_id = (self.template.strategy.convert_login_id_to_center_id)(login_id);
            let mut message = self
                .template
                .build_signout_message(center_id, self.session.device_id()?.as_deref());
            let response = self.template.push_message(&mut message)?;
            ensure_success_response(&response, SaSsoErrorCode::CODE_30006)?;
            self.session.logout()?;
        }
        Ok(SaSsoProcessorHelper::sso_logout_back(
            request,
            &self.template.common.param_name,
        ))
    }

    fn sso_push(&self, request: &SaSsoRequest) -> Result<SaSsoProcessorResult, SaSsoException> {
        if self.template.config().is_check_sign {
            self.template
                .sign_template()
                .check_param_map(&request.params)
                .map_err(protocol_error)?;
        }
        let message = message_from_params(&request.params);
        self.template
            .handle_message(&message)
            .map(SaSsoProcessorResult::Json)
    }

    fn sso_logout_call(
        &self,
        request: &SaSsoRequest,
    ) -> Result<SaSsoProcessorResult, SaSsoException> {
        if self.template.config().is_check_sign {
            self.template
                .sign_template()
                .check_param_map(&request.params)
                .map_err(protocol_error)?;
        }
        let params = &self.template.common.param_name;
        let center_id = request
            .param(&params.login_id)
            .ok_or_else(|| {
                SaSsoException::new(
                    SaSsoErrorCode::CODE_30024,
                    format!("missing parameter: {}", params.login_id),
                )
            })
            .map(|value| Value::String(value.to_owned()))?;
        let login_id = (self.template.strategy.convert_center_id_to_login_id)(center_id);
        let mut message = SaSsoMessage::with_type(SaSsoConsts::MESSAGE_LOGOUT_CALL)
            .set(params.login_id.clone(), login_id);
        if let Some(device_id) = request.param(&params.device_id) {
            message = message.set(params.device_id.clone(), device_id.to_owned());
        }
        self.template
            .handle_message(&message)
            .map(SaSsoProcessorResult::Json)
    }

    /// Checks a ticket using HTTP mode or the explicit direct server.
    ///
    /// # Errors
    ///
    /// Returns ticket, transport, response, or configuration failures.
    pub fn check_ticket(
        &self,
        ticket: &str,
        current_uri: Option<&str>,
    ) -> Result<SaCheckTicketResult, SaSsoException> {
        if self.template.config().is_http {
            let callback = if self.template.config().reg_logout_call {
                self.template
                    .config()
                    .curr_sso_logout_call
                    .as_deref()
                    .or(current_uri)
            } else {
                None
            };
            let mut message = self.template.build_check_ticket_message(ticket, callback);
            let response = self.template.push_message(&mut message)?;
            let value = ensure_success_response(&response, SaSsoErrorCode::CODE_30005)?;
            check_result_from_value(
                value,
                &self.template.common.param_name,
                &self.template.strategy,
            )
        } else {
            let server = self.direct_server.as_ref().ok_or_else(|| {
                SaSsoException::new(
                    SaSsoErrorCode::CODE_30005,
                    "direct SSO server template is not configured",
                )
            })?;
            let model = server
                .check_ticket_and_delete(ticket, self.template.client().unwrap_or_default())?;
            let center_id = model.login_id;
            Ok(SaCheckTicketResult {
                login_id: Some((self.template.strategy.convert_center_id_to_login_id)(
                    center_id.clone(),
                )),
                token_value: Some(model.token_value),
                center_id: Some(center_id),
                ..Default::default()
            })
        }
    }
}

fn message_from_params(params: &HashMap<String, String>) -> SaSsoMessage {
    SaSsoMessage::from_map(
        params
            .iter()
            .map(|(key, value)| (key.clone(), Value::String(value.clone())))
            .collect::<Map<_, _>>(),
    )
}

fn ensure_success_response(body: &str, error_code: i32) -> Result<Value, SaSsoException> {
    let value: Value = serde_json::from_str(body).map_err(protocol_error)?;
    if value.get("code").and_then(Value::as_i64) == Some(200) {
        Ok(value)
    } else {
        Err(SaSsoException::new(
            error_code,
            value
                .get("msg")
                .and_then(Value::as_str)
                .unwrap_or("SSO request failed"),
        ))
    }
}

fn check_result_from_value(
    value: Value,
    params: &crate::sso::name::ParamName,
    strategy: &SaSsoClientStrategy,
) -> Result<SaCheckTicketResult, SaSsoException> {
    let center_id = value.get(&params.login_id).cloned().ok_or_else(|| {
        SaSsoException::new(
            SaSsoErrorCode::CODE_30005,
            "check-ticket response has no loginId",
        )
    })?;
    Ok(SaCheckTicketResult {
        login_id: Some((strategy.convert_center_id_to_login_id)(center_id.clone())),
        token_value: value
            .get(&params.token_value)
            .and_then(Value::as_str)
            .map(ToOwned::to_owned),
        device_id: value
            .get(&params.device_id)
            .and_then(Value::as_str)
            .map(ToOwned::to_owned),
        remain_token_timeout: value
            .get(&params.remain_token_timeout)
            .and_then(Value::as_i64),
        remain_session_timeout: value
            .get(&params.remain_session_timeout)
            .and_then(Value::as_i64),
        center_id: Some(center_id),
        result: Some(value),
    })
}

fn protocol_error(error: impl std::fmt::Display) -> SaSsoException {
    SaSsoException::new(SaSsoErrorCode::CODE_30005, error.to_string())
}
