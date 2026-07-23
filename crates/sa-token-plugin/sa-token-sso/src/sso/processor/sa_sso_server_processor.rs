use crate::sso::error::SaSsoErrorCode;
use crate::sso::exception::SaSsoException;
use crate::sso::message::SaSsoMessage;
use crate::sso::processor::{SaSsoProcessorHelper, SaSsoProcessorResult, SaSsoRequest};
use crate::sso::template::SaSsoServerTemplate;
use crate::sso::util::SaSsoConsts;
use serde_json::{Map, Value, json};
use std::sync::Arc;

/// Current-request session operations required by the SSO server processor.
pub trait SaSsoServerSession: Send + Sync + 'static {
    /// Returns `(login_id, token_value, device_id)` for the current request.
    fn current_login(&self) -> Result<Option<(Value, String, Option<String>)>, SaSsoException>;

    /// Renews the current token lifetime after a successful authorization.
    fn renew_timeout(&self) -> Result<(), SaSsoException>;
}

/// Framework-neutral SSO server route processor.
pub struct SaSsoServerProcessor {
    pub template: Arc<SaSsoServerTemplate>,
    session: Arc<dyn SaSsoServerSession>,
}

impl SaSsoServerProcessor {
    /// Creates a server processor.
    pub fn new(template: Arc<SaSsoServerTemplate>, session: Arc<dyn SaSsoServerSession>) -> Self {
        Self { template, session }
    }

    /// Dispatches all SSO server routes.
    ///
    /// # Errors
    ///
    /// Returns redirect, signing, ticket, auth, or protocol failures.
    pub fn dispatch(&self, request: &SaSsoRequest) -> Result<SaSsoProcessorResult, SaSsoException> {
        let api = &self.template.common.api_name;
        if request.path == api.sso_auth {
            self.sso_auth(request)
        } else if request.path == api.sso_do_login {
            Ok(self.sso_do_login(request))
        } else if request.path == api.sso_signout {
            self.sso_signout(request)
        } else if request.path == api.sso_push_s {
            self.sso_push(request)
        } else {
            Ok(SaSsoProcessorResult::NotHandled)
        }
    }

    fn sso_auth(&self, request: &SaSsoRequest) -> Result<SaSsoProcessorResult, SaSsoException> {
        let Some((login_id, token_value, _)) = self.session.current_login()? else {
            return Ok(SaSsoProcessorResult::Json((self
                .template
                .strategy
                .not_login_view)(
            )));
        };
        let params = &self.template.common.param_name;
        let client = request.param(&params.client).unwrap_or_default();
        let redirect = match request.param(&params.redirect) {
            Some(redirect) if !redirect.is_empty() => redirect,
            _ => {
                return self
                    .template
                    .config()
                    .home_route
                    .clone()
                    .map(SaSsoProcessorResult::Redirect)
                    .ok_or_else(|| {
                        SaSsoException::new(
                            SaSsoErrorCode::CODE_30014,
                            "redirect and homeRoute are both missing",
                        )
                    });
            }
        };
        let mode = request
            .param(&params.mode)
            .unwrap_or(SaSsoConsts::MODE_TICKET);
        let final_redirect = if mode == SaSsoConsts::MODE_SIMPLE {
            self.template.check_redirect_url(client, redirect)?;
            redirect.to_owned()
        } else {
            self.template
                .build_redirect_url(client, redirect, login_id, &token_value)?
        };
        if self.template.config().auto_renew_timeout {
            self.session.renew_timeout()?;
        }
        (self.template.strategy.jump_to_redirect_url_notice)(&final_redirect);
        Ok(SaSsoProcessorResult::Redirect(final_redirect))
    }

    fn sso_do_login(&self, request: &SaSsoRequest) -> SaSsoProcessorResult {
        let params = &self.template.common.param_name;
        SaSsoProcessorResult::Json((self.template.strategy.do_login_handle)(
            request.param(&params.name).unwrap_or_default(),
            request.param(&params.pwd).unwrap_or_default(),
        ))
    }

    fn sso_signout(&self, request: &SaSsoRequest) -> Result<SaSsoProcessorResult, SaSsoException> {
        if let Some((login_id, _, current_device)) = self.session.current_login()? {
            let single_device = request
                .param(&self.template.common.param_name.single_device_id_logout)
                == Some("true");
            self.template
                .sso_logout(&login_id, single_device.then_some(current_device).flatten())?;
        }
        Ok(SaSsoProcessorHelper::sso_logout_back(
            request,
            &self.template.common.param_name,
        ))
    }

    fn sso_push(&self, request: &SaSsoRequest) -> Result<SaSsoProcessorResult, SaSsoException> {
        let params = &self.template.common.param_name;
        let client = request.param(&params.client).unwrap_or_default();
        if client == SaSsoConsts::CLIENT_WILDCARD {
            return Ok(SaSsoProcessorResult::Json(
                json!({"code": 500, "msg": format!("invalid client: {client}")}),
            ));
        }
        if self.template.config().is_check_sign {
            self.template
                .sign_template(client)?
                .check_param_map(&request.params)
                .map_err(protocol_error)?;
        }
        let message = SaSsoMessage::from_map(
            request
                .params
                .iter()
                .map(|(key, value)| (key.clone(), Value::String(value.clone())))
                .collect::<Map<_, _>>(),
        );
        if !self
            .template
            .common
            .message_holder
            .has_handle(message.get_type().unwrap_or_default())
        {
            return Ok(SaSsoProcessorResult::Json(json!({
                "code": 500,
                "msg": format!(
                    "SSO message handler not found: {}",
                    message.get_type().unwrap_or_default()
                )
            })));
        }
        self.template
            .handle_message(&message)
            .map(SaSsoProcessorResult::Json)
    }
}

fn protocol_error(error: impl std::fmt::Display) -> SaSsoException {
    SaSsoException::new(SaSsoErrorCode::CODE_30001, error.to_string())
}
