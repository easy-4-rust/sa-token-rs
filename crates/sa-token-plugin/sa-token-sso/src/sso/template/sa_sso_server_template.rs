use crate::sso::config::{SaSsoClientModel, SaSsoServerConfig};
use crate::sso::error::SaSsoErrorCode;
use crate::sso::exception::SaSsoException;
use crate::sso::message::{SaSsoMessage, SaSsoMessageCheckTicketHandle, SaSsoMessageSignoutHandle};
use crate::sso::model::{SaSsoClientInfo, TicketModel};
use crate::sso::strategy::SaSsoServerStrategy;
use crate::sso::template::SaSsoTemplate;
use crate::sso::util::SaSsoConsts;
use sa_token_core::dao::sa_token_dao::SaTokenDao;
use sa_token_core::util::sa_fox_util::random_string;
use sa_token_sign::sign::{SaSignConfig, SaSignTemplate};
use serde_json::{Value, json};
use std::sync::Arc;
use url::Url;

/// Authentication/session operations required by the SSO server.
pub trait SaSsoServerAuth: Send + Sync + 'static {
    /// Returns the device ID bound to a token.
    fn login_device_id_by_token(&self, token_value: &str)
    -> Result<Option<String>, SaSsoException>;

    /// Returns the remaining token lifetime in seconds.
    fn token_timeout(&self, token_value: &str) -> Result<i64, SaSsoException>;

    /// Returns the remaining account-session lifetime in seconds.
    fn session_timeout(&self, login_id: &Value) -> Result<i64, SaSsoException>;

    /// Invalidates the requested account/device session.
    fn logout(&self, login_id: &Value, device_id: Option<String>) -> Result<(), SaSsoException>;
}

struct ServerState {
    config: Arc<SaSsoServerConfig>,
    strategy: Arc<SaSsoServerStrategy>,
    sign_config: Arc<SaSignConfig>,
    dao: Arc<dyn SaTokenDao>,
    auth: Arc<dyn SaSsoServerAuth>,
    token_name: String,
}

/// SSO server protocol operations with explicit DAO and auth runtime ports.
pub struct SaSsoServerTemplate {
    pub common: SaSsoTemplate,
    pub strategy: Arc<SaSsoServerStrategy>,
    state: Arc<ServerState>,
}

impl SaSsoServerTemplate {
    /// Creates an isolated server template and installs the Java message
    /// handlers.
    ///
    /// # Errors
    ///
    /// Returns an error when the handler registry cannot be updated.
    pub fn new(
        config: Arc<SaSsoServerConfig>,
        strategy: Arc<SaSsoServerStrategy>,
        sign_config: Arc<SaSignConfig>,
        dao: Arc<dyn SaTokenDao>,
        auth: Arc<dyn SaSsoServerAuth>,
        token_name: impl Into<String>,
    ) -> Result<Self, SaSsoException> {
        let state = Arc::new(ServerState {
            config,
            strategy: Arc::clone(&strategy),
            sign_config,
            dao,
            auth,
            token_name: token_name.into(),
        });
        let common = SaSsoTemplate::new();

        let check_state = Arc::clone(&state);
        let check_params = common.param_name.clone();
        let check_callback = Arc::new(move |_: &SaSsoTemplate, message: &SaSsoMessage| {
            let client = message
                .get(&check_params.client)
                .and_then(Value::as_str)
                .unwrap_or_default();
            let ticket = required_string(message, &check_params.ticket)?;
            let slo_callback = message
                .get(&check_params.sso_logout_call)
                .and_then(Value::as_str);
            let model = check_ticket_and_delete(&check_state, &ticket, client)?;
            register_slo_callback(
                &check_state,
                &model.login_id,
                client,
                slo_callback.unwrap_or_default(),
            )?;
            let device_id = check_state
                .auth
                .login_device_id_by_token(&model.token_value)?;
            let token_timeout = check_state.auth.token_timeout(&model.token_value)?;
            let session_timeout = check_state.auth.session_timeout(&model.login_id)?;
            let mut result = json!({
                "code": 200,
                "msg": "ok",
                "data": model.login_id,
                check_params.login_id.clone(): model.login_id,
                check_params.token_value.clone(): model.token_value,
                check_params.device_id.clone(): device_id,
                check_params.remain_token_timeout.clone(): token_timeout,
                check_params.remain_session_timeout.clone(): session_timeout
            });
            result =
                (check_state.strategy.check_ticket_append_data)(model.login_id.clone(), result);
            Ok(result)
        });
        common
            .message_holder
            .add_handle(Arc::new(SaSsoMessageCheckTicketHandle::new(check_callback)))?;

        let signout_state = Arc::clone(&state);
        let signout_params = common.param_name.clone();
        let signout_callback = Arc::new(move |_: &SaSsoTemplate, message: &SaSsoMessage| {
            if !signout_state.config.is_slo {
                return Ok(json!({
                    "code": 500,
                    "msg": "当前 sso-server 端未开启单点注销功能"
                }));
            }
            let login_id = message
                .get_value_not_null(&signout_params.login_id)?
                .clone();
            let device_id = message
                .get(&signout_params.device_id)
                .and_then(Value::as_str)
                .map(ToOwned::to_owned);
            signout_state.auth.logout(&login_id, device_id)?;
            Ok(json!({"code": 200, "msg": "ok"}))
        });
        common
            .message_holder
            .add_handle(Arc::new(SaSsoMessageSignoutHandle::new(signout_callback)))?;

        Ok(Self {
            common,
            strategy,
            state,
        })
    }

    /// Returns the immutable server configuration.
    pub fn config(&self) -> &Arc<SaSsoServerConfig> {
        &self.state.config
    }

    /// Builds a ticket model without persisting it.
    pub fn create_ticket(
        &self,
        client: impl Into<String>,
        login_id: Value,
        token_value: impl Into<String>,
    ) -> TicketModel {
        TicketModel::new(random_string(64), client, login_id, token_value)
    }

    /// Persists a ticket and its account index.
    ///
    /// # Errors
    ///
    /// Returns serialization or DAO failures.
    pub fn save_ticket(&self, model: &TicketModel) -> Result<(), SaSsoException> {
        let value = serde_json::to_value(model).map_err(protocol_error)?;
        self.state
            .dao
            .set_object(
                &self.ticket_key(&model.ticket),
                &value,
                self.state.config.ticket_timeout,
            )
            .map_err(protocol_error)?;
        self.state
            .dao
            .set(
                &self.ticket_index_key(&model.client, &model.login_id),
                &model.ticket,
                self.state.config.ticket_timeout,
            )
            .map_err(protocol_error)
    }

    /// Creates and persists a one-time ticket.
    ///
    /// # Errors
    ///
    /// Returns serialization or DAO failures.
    pub fn create_ticket_and_save(
        &self,
        client: impl Into<String>,
        login_id: Value,
        token_value: impl Into<String>,
    ) -> Result<String, SaSsoException> {
        let model = self.create_ticket(client, login_id, token_value);
        self.save_ticket(&model)?;
        Ok(model.ticket)
    }

    /// Loads a ticket without consuming it.
    ///
    /// # Errors
    ///
    /// Returns DAO or deserialization failures.
    pub fn get_ticket(&self, ticket: &str) -> Result<Option<TicketModel>, SaSsoException> {
        if ticket.is_empty() {
            return Ok(None);
        }
        self.state
            .dao
            .get_object(&self.ticket_key(ticket))
            .map_err(protocol_error)?
            .map(serde_json::from_value)
            .transpose()
            .map_err(protocol_error)
    }

    /// Validates client ownership and consumes a ticket.
    ///
    /// # Errors
    ///
    /// Returns code `30004` for an unknown ticket, code `30011` for a client
    /// mismatch, or a DAO failure.
    pub fn check_ticket_and_delete(
        &self,
        ticket: &str,
        client: &str,
    ) -> Result<TicketModel, SaSsoException> {
        check_ticket_and_delete(&self.state, ticket, client)
    }

    /// Deletes a ticket and its index.
    ///
    /// # Errors
    ///
    /// Returns DAO failures.
    pub fn delete_ticket(&self, model: &TicketModel) -> Result<(), SaSsoException> {
        self.state
            .dao
            .delete_object(&self.ticket_key(&model.ticket))
            .map_err(protocol_error)?;
        self.state
            .dao
            .delete(&self.ticket_index_key(&model.client, &model.login_id))
            .map_err(protocol_error)
    }

    /// Deletes a ticket by value when it exists.
    ///
    /// # Errors
    ///
    /// Returns DAO or deserialization failures.
    pub fn delete_ticket_value(&self, ticket: &str) -> Result<(), SaSsoException> {
        if let Some(model) = self.get_ticket(ticket)? {
            self.delete_ticket(&model)?;
        }
        Ok(())
    }

    /// Returns the login ID referenced by a ticket.
    ///
    /// # Errors
    ///
    /// Returns DAO or deserialization failures.
    pub fn login_id(&self, ticket: &str) -> Result<Option<Value>, SaSsoException> {
        Ok(self.get_ticket(ticket)?.map(|model| model.login_id))
    }

    /// Returns the indexed ticket for a client/account pair.
    ///
    /// # Errors
    ///
    /// Returns DAO failures.
    pub fn ticket_value(
        &self,
        client: &str,
        login_id: &Value,
    ) -> Result<Option<String>, SaSsoException> {
        self.state
            .dao
            .get(&self.ticket_index_key(client, login_id))
            .map_err(protocol_error)
    }

    /// Registers a mode-three logout callback.
    ///
    /// # Errors
    ///
    /// Returns DAO or serialization failures.
    pub fn register_slo_callback(
        &self,
        login_id: &Value,
        client: &str,
        callback: &str,
    ) -> Result<(), SaSsoException> {
        register_slo_callback(&self.state, login_id, client, callback)
    }

    /// Invalidates the server-side account/device session.
    ///
    /// # Errors
    ///
    /// Returns the auth runtime failure.
    pub fn sso_logout(
        &self,
        login_id: &Value,
        device_id: Option<String>,
    ) -> Result<(), SaSsoException> {
        self.state.auth.logout(login_id, device_id)
    }

    /// Returns a configured client, including the anonymous fallback.
    ///
    /// # Errors
    ///
    /// Returns code `30013` for an unknown named client.
    pub fn client(&self, client: &str) -> Result<SaSsoClientModel, SaSsoException> {
        client_from_config(&self.state.config, &self.state.sign_config, client)
    }

    /// Returns clients configured for push delivery.
    pub fn push_clients(&self) -> Vec<SaSsoClientModel> {
        self.state
            .config
            .clients
            .values()
            .filter(|client| client.is_push)
            .cloned()
            .collect()
    }

    /// Validates a redirect against the selected client's allow list.
    ///
    /// # Errors
    ///
    /// Returns Java-compatible codes `30001`, `30002`, `30013`, or `30015`.
    pub fn check_redirect_url(&self, client: &str, redirect: &str) -> Result<(), SaSsoException> {
        Url::parse(redirect).map_err(|_| {
            SaSsoException::new(
                SaSsoErrorCode::CODE_30001,
                format!("invalid redirect: {redirect}"),
            )
        })?;
        let without_query = redirect.split('?').next().unwrap_or(redirect);
        if without_query.contains('@') {
            return Err(SaSsoException::new(
                SaSsoErrorCode::CODE_30001,
                format!("redirect must not contain @: {without_query}"),
            ));
        }
        let model = self.client(client)?;
        let allowed = parse_allow_urls(&model.allow_url)?;
        if allowed
            .iter()
            .any(|rule| wildcard_match(rule, without_query))
        {
            Ok(())
        } else {
            Err(SaSsoException::new(
                SaSsoErrorCode::CODE_30002,
                format!("redirect is not allowed: {without_query}"),
            ))
        }
    }

    /// Builds a validated redirect URL and rotates any previous ticket.
    ///
    /// # Errors
    ///
    /// Returns redirect validation, serialization, or DAO failures.
    pub fn build_redirect_url(
        &self,
        client: &str,
        redirect: &str,
        login_id: Value,
        token_value: &str,
    ) -> Result<String, SaSsoException> {
        self.check_redirect_url(client, redirect)?;
        let index_key = self.ticket_index_key(client, &login_id);
        if let Some(old_ticket) = self.state.dao.get(&index_key).map_err(protocol_error)?
            && let Some(old_model) = self.get_ticket(&old_ticket)?
        {
            self.delete_ticket(&old_model)?;
        }
        let ticket =
            self.create_ticket_and_save(client.to_owned(), login_id, token_value.to_owned())?;
        append_pair(
            &self.encode_back_param(redirect),
            &self.common.param_name.ticket,
            &ticket,
        )
    }

    /// Percent-encodes the complete trailing `back` value, matching Java's
    /// nested-query handling.
    pub fn encode_back_param(&self, url: &str) -> String {
        let question = format!("?{}=", self.common.param_name.back);
        let ampersand = format!("&{}=", self.common.param_name.back);
        let Some(index) = url.find(&question).or_else(|| url.find(&ampersand)) else {
            return url.to_owned();
        };
        let value_start = index + self.common.param_name.back.len() + 2;
        let encoded: String =
            url::form_urlencoded::byte_serialize(&url.as_bytes()[value_start..]).collect();
        format!("{}{encoded}", &url[..value_start])
    }

    /// Returns the signing template using client-over-server-over-global
    /// secret precedence.
    ///
    /// # Errors
    ///
    /// Returns client lookup failures.
    pub fn sign_template(&self, client: &str) -> Result<SaSignTemplate, SaSsoException> {
        let model = self.client(client)?;
        let mut config = (*self.state.sign_config).clone();
        if let Some(secret) = model
            .secret_key
            .as_deref()
            .filter(|secret| !secret.is_empty())
            .or_else(|| {
                self.state
                    .config
                    .secret_key
                    .as_deref()
                    .filter(|secret| !secret.is_empty())
            })
        {
            config.secret_key = secret.to_owned();
        }
        Ok(SaSignTemplate::new(
            Arc::new(config),
            Arc::clone(&self.state.dao),
            &self.state.token_name,
        ))
    }

    /// Dispatches an incoming server message.
    ///
    /// # Errors
    ///
    /// Returns message validation, ticket, auth, or handler errors.
    pub fn handle_message(&self, message: &SaSsoMessage) -> Result<Value, SaSsoException> {
        self.common.handle_message(message)
    }

    /// Returns the persistent ticket key.
    pub fn ticket_key(&self, ticket: &str) -> String {
        format!("{}:ticket:{ticket}", self.state.token_name)
    }

    /// Returns the persistent account-to-ticket index key.
    pub fn ticket_index_key(&self, client: &str, login_id: &Value) -> String {
        let client = if client.is_empty() || client == SaSsoConsts::CLIENT_WILDCARD {
            SaSsoConsts::CLIENT_ANON
        } else {
            client
        };
        format!(
            "{}:ticket-index:{client}:{}",
            self.state.token_name,
            value_text(login_id)
        )
    }
}

fn check_ticket_and_delete(
    state: &ServerState,
    ticket: &str,
    client: &str,
) -> Result<TicketModel, SaSsoException> {
    let key = format!("{}:ticket:{ticket}", state.token_name);
    let model: TicketModel = state
        .dao
        .get_object(&key)
        .map_err(protocol_error)?
        .ok_or_else(|| {
            SaSsoException::new(
                SaSsoErrorCode::CODE_30004,
                format!("invalid ticket: {ticket}"),
            )
        })
        .and_then(|value| serde_json::from_value(value).map_err(protocol_error))?;
    if client != SaSsoConsts::CLIENT_WILDCARD
        && !(client.is_empty() && model.client.is_empty())
        && client != model.client
    {
        return Err(SaSsoException::new(
            SaSsoErrorCode::CODE_30011,
            format!("ticket does not belong to client={client}: {ticket}"),
        ));
    }
    state.dao.delete_object(&key).map_err(protocol_error)?;
    let index_client = if model.client.is_empty() {
        SaSsoConsts::CLIENT_ANON
    } else {
        &model.client
    };
    let index_key = format!(
        "{}:ticket-index:{index_client}:{}",
        state.token_name,
        value_text(&model.login_id)
    );
    state.dao.delete(&index_key).map_err(protocol_error)?;
    Ok(model)
}

fn register_slo_callback(
    state: &ServerState,
    login_id: &Value,
    client: &str,
    callback: &str,
) -> Result<(), SaSsoException> {
    if login_id.is_null() {
        return Ok(());
    }
    let key = format!("{}:sso-client:{}", state.token_name, value_text(login_id));
    let mut clients: Vec<SaSsoClientInfo> = state
        .dao
        .get_object(&key)
        .map_err(protocol_error)?
        .map(serde_json::from_value)
        .transpose()
        .map_err(protocol_error)?
        .unwrap_or_default();
    let index = clients
        .last()
        .map(|client| client.index.checked_add(1).unwrap_or(0))
        .unwrap_or(0);
    clients.push(SaSsoClientInfo::mode_three(client, callback, index));
    if state.config.max_reg_client >= 0 {
        let max = state.config.max_reg_client as usize;
        if clients.len() > max {
            clients.drain(..clients.len() - max);
        }
    }
    let value = serde_json::to_value(clients).map_err(protocol_error)?;
    state
        .dao
        .set_object(&key, &value, -1)
        .map_err(protocol_error)
}

fn client_from_config(
    config: &SaSsoServerConfig,
    sign_config: &SaSignConfig,
    client: &str,
) -> Result<SaSsoClientModel, SaSsoException> {
    if client.is_empty() {
        if !config.allow_anon_client {
            return Err(SaSsoException::new(0, "client must not be empty"));
        }
        return Ok(SaSsoClientModel {
            allow_url: config.allow_url.clone(),
            is_slo: config.is_slo,
            secret_key: config
                .secret_key
                .clone()
                .or_else(|| Some(sign_config.secret_key.clone())),
            ..Default::default()
        });
    }
    config.clients.get(client).cloned().ok_or_else(|| {
        SaSsoException::new(
            SaSsoErrorCode::CODE_30013,
            format!("unknown SSO client: {client}"),
        )
    })
}

fn parse_allow_urls(value: &str) -> Result<Vec<String>, SaSsoException> {
    value
        .replace(' ', "")
        .split(',')
        .map(ToOwned::to_owned)
        .map(|rule| {
            if let Some(index) = rule.find('*')
                && index != rule.len() - 1
            {
                return Err(SaSsoException::new(
                    SaSsoErrorCode::CODE_30015,
                    format!("wildcard is only allowed at the end: {rule}"),
                ));
            }
            Ok(rule)
        })
        .collect()
}

fn wildcard_match(rule: &str, value: &str) -> bool {
    rule.strip_suffix('*')
        .map_or(rule == value, |prefix| value.starts_with(prefix))
}

fn required_string(message: &SaSsoMessage, key: &str) -> Result<String, SaSsoException> {
    let value = message.get_value_not_null(key)?;
    match value {
        Value::String(value) if !value.is_empty() => Ok(value.clone()),
        Value::Null => Err(SaSsoException::new(
            SaSsoErrorCode::CODE_30024,
            format!("missing parameter: {key}"),
        )),
        value => Ok(value.to_string()),
    }
}

fn value_text(value: &Value) -> String {
    match value {
        Value::String(value) => value.clone(),
        value => value.to_string(),
    }
}

fn append_pair(base: &str, key: &str, value: &str) -> Result<String, SaSsoException> {
    let mut url = Url::parse(base).map_err(|error| {
        SaSsoException::new(
            SaSsoErrorCode::CODE_30001,
            format!("invalid redirect URL: {error}"),
        )
    })?;
    url.query_pairs_mut().append_pair(key, value);
    Ok(url.into())
}

fn protocol_error(error: impl std::fmt::Display) -> SaSsoException {
    SaSsoException::new(SaSsoErrorCode::CODE_30001, error.to_string())
}
