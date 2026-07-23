use crate::sso::config::SaSsoClientConfig;
use crate::sso::error::SaSsoErrorCode;
use crate::sso::exception::SaSsoException;
use crate::sso::message::{SaSsoMessage, SaSsoMessageLogoutCallHandle};
use crate::sso::strategy::SaSsoClientStrategy;
use crate::sso::template::SaSsoTemplate;
use crate::sso::util::SaSsoConsts;
use sa_token_core::dao::sa_token_dao::SaTokenDao;
use sa_token_sign::sign::{SaSignConfig, SaSignTemplate};
use serde_json::{Value, json};
use std::collections::HashMap;
use std::sync::Arc;
use url::Url;

/// Runtime port used to invalidate a local client session.
pub type SaSsoClientLogoutFunction =
    Arc<dyn Fn(Value, Option<String>) -> Result<(), SaSsoException> + Send + Sync + 'static>;

/// SSO client protocol operations with explicit runtime dependencies.
pub struct SaSsoClientTemplate {
    pub common: SaSsoTemplate,
    pub strategy: Arc<SaSsoClientStrategy>,
    config: Arc<SaSsoClientConfig>,
    sign_config: Arc<SaSignConfig>,
    dao: Arc<dyn SaTokenDao>,
    token_name: String,
}

impl SaSsoClientTemplate {
    /// Creates an isolated client template and installs its logout handler.
    ///
    /// # Errors
    ///
    /// Returns an error when the handler registry cannot be updated.
    pub fn new(
        config: Arc<SaSsoClientConfig>,
        strategy: Arc<SaSsoClientStrategy>,
        sign_config: Arc<SaSignConfig>,
        dao: Arc<dyn SaTokenDao>,
        token_name: impl Into<String>,
        logout: SaSsoClientLogoutFunction,
    ) -> Result<Self, SaSsoException> {
        let common = SaSsoTemplate::new();
        let param_name = common.param_name.clone();
        let config_for_handler = Arc::clone(&config);
        let strategy_for_handler = Arc::clone(&strategy);
        let callback = Arc::new(move |_: &SaSsoTemplate, message: &SaSsoMessage| {
            if !config_for_handler.is_slo {
                return Ok(json!({
                    "code": 500,
                    "msg": "当前 sso-client 端未开启单点注销功能"
                }));
            }
            let center_id = message.get_value_not_null(&param_name.login_id)?.clone();
            let login_id = (strategy_for_handler.convert_center_id_to_login_id)(center_id);
            let device_id = message
                .get(&param_name.device_id)
                .and_then(Value::as_str)
                .map(ToOwned::to_owned);
            logout(login_id, device_id)?;
            Ok(json!({"code": 200, "msg": "单点注销回调成功"}))
        });
        common
            .message_holder
            .add_handle(Arc::new(SaSsoMessageLogoutCallHandle::new(callback)))?;
        Ok(Self {
            common,
            strategy,
            config,
            sign_config,
            dao,
            token_name: token_name.into(),
        })
    }

    /// Returns the immutable client configuration.
    pub fn config(&self) -> &Arc<SaSsoClientConfig> {
        &self.config
    }

    /// Returns the configured client identifier.
    pub fn client(&self) -> Option<&str> {
        self.config.client.as_deref()
    }

    /// Creates the signing template using SSO-over-global secret precedence.
    pub fn sign_template(&self) -> SaSignTemplate {
        let mut config = (*self.sign_config).clone();
        if let Some(secret) = self
            .config
            .secret_key
            .as_deref()
            .filter(|secret| !secret.is_empty())
        {
            config.secret_key = secret.to_owned();
        }
        SaSignTemplate::new(Arc::new(config), Arc::clone(&self.dao), &self.token_name)
    }

    /// Builds a signed server URL for a configured or absolute path.
    ///
    /// # Errors
    ///
    /// Returns code `30012` when a relative path has no server URL, or a
    /// signing/URL error.
    pub fn build_custom_path_url(
        &self,
        path: &str,
        params: &HashMap<String, Value>,
    ) -> Result<String, SaSsoException> {
        let base = if path.starts_with("http") {
            path.to_owned()
        } else {
            let server_url = self
                .config
                .server_url
                .as_deref()
                .filter(|value| !value.is_empty())
                .ok_or_else(|| {
                    SaSsoException::new(
                        SaSsoErrorCode::CODE_30012,
                        "sa-token.sso-client.server-url is required",
                    )
                })?;
            splice_url(server_url, path)
        };
        let mut string_params = value_map_to_strings_from_hash(params);
        if let Some(client) = self.client() {
            string_params.insert(self.common.param_name.client.clone(), client.to_owned());
        }
        self.sign_template()
            .add_sign_params(&mut string_params)
            .map_err(sign_error)?;
        append_query(&base, &string_params)
    }

    /// Sends a signed custom data request.
    ///
    /// # Errors
    ///
    /// Returns URL/signing or transport errors.
    pub fn get_data(
        &self,
        path: &str,
        params: &HashMap<String, Value>,
    ) -> Result<String, SaSsoException> {
        let url = self.build_custom_path_url(path, params)?;
        (self.strategy.send_request)(&url)
    }

    /// Builds the server authorization URL, preserving an existing back
    /// parameter on the client callback.
    pub fn build_server_auth_url(
        &self,
        client_login_url: &str,
        back: Option<&str>,
    ) -> Result<String, SaSsoException> {
        let mut server_url = self.config.splicing_auth_url();
        if let Some(client) = self.client().filter(|value| !value.is_empty()) {
            server_url = append_pair_raw(&server_url, &self.common.param_name.client, client);
        }
        let callback = if client_login_url.contains(&format!("{}=", self.common.param_name.back)) {
            client_login_url.to_owned()
        } else {
            append_pair_raw(
                client_login_url,
                &self.common.param_name.back,
                &percent_encode(back.unwrap_or_default()),
            )
        };
        Ok(append_pair_raw(
            &server_url,
            &self.common.param_name.redirect,
            &callback,
        ))
    }

    /// Pushes a signed protocol message to the SSO server.
    ///
    /// # Errors
    ///
    /// Returns code `30023` for an invalid push URL, or a validation,
    /// signing, or transport error.
    pub fn push_message(&self, message: &mut SaSsoMessage) -> Result<String, SaSsoException> {
        message.check_type()?;
        if let Some(client) = self.client() {
            message.insert(
                self.common.param_name.client.clone(),
                Value::String(client.to_owned()),
            );
        }
        let push_url = self.config.splicing_push_url();
        Url::parse(&push_url).map_err(|_| {
            SaSsoException::new(
                SaSsoErrorCode::CODE_30023,
                format!("invalid push URL: {push_url}"),
            )
        })?;
        let mut params = value_map_to_strings(message.as_map());
        self.sign_template()
            .add_sign_params(&mut params)
            .map_err(sign_error)?;
        let final_url = append_query(&push_url, &params)?;
        (self.strategy.send_request)(&final_url)
    }

    /// Builds a check-ticket message.
    pub fn build_check_ticket_message(
        &self,
        ticket: impl Into<String>,
        slo_callback_url: Option<&str>,
    ) -> SaSsoMessage {
        let mut message = SaSsoMessage::with_type(SaSsoConsts::MESSAGE_CHECK_TICKET)
            .set(self.common.param_name.ticket.clone(), ticket.into());
        if let Some(client) = self.client() {
            message = message.set(self.common.param_name.client.clone(), client.to_owned());
        }
        if let Some(callback) = slo_callback_url {
            message = message.set(
                self.common.param_name.sso_logout_call.clone(),
                callback.to_owned(),
            );
        }
        message
    }

    /// Builds a sign-out message.
    pub fn build_signout_message(&self, login_id: Value, device_id: Option<&str>) -> SaSsoMessage {
        let mut message = SaSsoMessage::with_type(SaSsoConsts::MESSAGE_SIGNOUT)
            .set(self.common.param_name.login_id.clone(), login_id);
        if let Some(client) = self.client() {
            message = message.set(self.common.param_name.client.clone(), client.to_owned());
        }
        if let Some(device_id) = device_id {
            message = message.set(
                self.common.param_name.device_id.clone(),
                device_id.to_owned(),
            );
        }
        message
    }

    /// Dispatches an incoming client message.
    ///
    /// # Errors
    ///
    /// Returns message validation or handler errors.
    pub fn handle_message(&self, message: &SaSsoMessage) -> Result<Value, SaSsoException> {
        self.common.handle_message(message)
    }
}

fn value_map_to_strings(params: &serde_json::Map<String, Value>) -> HashMap<String, String> {
    params
        .iter()
        .filter_map(|(key, value)| match value {
            Value::Null => None,
            Value::String(value) => Some((key.clone(), value.clone())),
            value => Some((key.clone(), value.to_string())),
        })
        .collect()
}

fn value_map_to_strings_from_hash(params: &HashMap<String, Value>) -> HashMap<String, String> {
    params
        .iter()
        .filter_map(|(key, value)| match value {
            Value::Null => None,
            Value::String(value) => Some((key.clone(), value.clone())),
            value => Some((key.clone(), value.to_string())),
        })
        .collect()
}

fn append_query(base: &str, params: &HashMap<String, String>) -> Result<String, SaSsoException> {
    let mut url = Url::parse(base).map_err(|error| {
        SaSsoException::new(
            SaSsoErrorCode::CODE_30023,
            format!("invalid SSO URL: {error}"),
        )
    })?;
    url.query_pairs_mut().extend_pairs(params.iter());
    Ok(url.into())
}

fn append_pair_raw(base: &str, key: &str, value: &str) -> String {
    let separator = if base.contains('?') { '&' } else { '?' };
    format!("{base}{separator}{key}={value}")
}

fn percent_encode(value: &str) -> String {
    url::form_urlencoded::byte_serialize(value.as_bytes()).collect()
}

fn splice_url(base: &str, path: &str) -> String {
    format!(
        "{}/{}",
        base.trim_end_matches('/'),
        path.trim_start_matches('/')
    )
}

fn sign_error(error: impl std::fmt::Display) -> SaSsoException {
    SaSsoException::new(SaSsoErrorCode::CODE_30001, error.to_string())
}
