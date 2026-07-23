use std::collections::BTreeMap;
use std::sync::{Arc, RwLock};

use sa_token_core::exception::{SaResult, SaTokenException};
use serde_json::Value;

use crate::oauth2::config::SaOAuth2ServerConfig;
use crate::oauth2::consts::{GrantType, SaOAuth2Consts, SaOAuth2Param};
use crate::oauth2::data::generate::SaOAuth2GenerateHooks;
use crate::oauth2::data::model::loader::SaClientModel;
use crate::oauth2::data::model::{AccessTokenModel, ClientTokenModel};
use crate::oauth2::data::resolver::{SaOAuth2DataResolver, SaOAuth2Request};
use crate::oauth2::function::{
    SaOAuth2ConfirmViewFunction, SaOAuth2DoLoginHandleFunction, SaOAuth2NotLoginViewFunction,
};
use crate::oauth2::grant_type::handler::SaOAuth2GrantTypeHandlerInterface;
use crate::oauth2::scope::handler::SaOAuth2ScopeHandlerInterface;

/// Validates client credentials and requested scopes before grant dispatch.
pub trait SaOAuth2ClientGrantValidator: Send + Sync {
    /// Returns the validated client registration.
    ///
    /// # Errors
    ///
    /// Returns client-secret or scope errors with their Java-compatible codes.
    fn validate(
        &self,
        client_id: &str,
        client_secret: &str,
        scopes: &[String],
    ) -> SaResult<SaClientModel>;
}

impl<F> SaOAuth2ClientGrantValidator for F
where
    F: Fn(&str, &str, &[String]) -> SaResult<SaClientModel> + Send + Sync,
{
    fn validate(
        &self,
        client_id: &str,
        client_secret: &str,
        scopes: &[String],
    ) -> SaResult<SaClientModel> {
        self(client_id, client_secret, scopes)
    }
}

type UserAuthorizeClientCheck = dyn Fn(&Value, &str) -> SaResult<()> + Send + Sync;

/// Isolated OAuth2 strategy registry replacing the Java process-wide singleton.
pub struct SaOAuth2Strategy {
    config: Arc<SaOAuth2ServerConfig>,
    resolver: Arc<dyn SaOAuth2DataResolver>,
    client_validator: Arc<dyn SaOAuth2ClientGrantValidator>,
    user_authorize_client_check: Arc<UserAuthorizeClientCheck>,
    not_login_view: Arc<dyn SaOAuth2NotLoginViewFunction>,
    confirm_view: Arc<dyn SaOAuth2ConfirmViewFunction>,
    do_login_handle: Arc<dyn SaOAuth2DoLoginHandleFunction>,
    scope_handlers: RwLock<BTreeMap<String, Arc<dyn SaOAuth2ScopeHandlerInterface>>>,
    grant_handlers: RwLock<BTreeMap<String, Arc<dyn SaOAuth2GrantTypeHandlerInterface>>>,
}

impl SaOAuth2Strategy {
    pub fn new(
        config: Arc<SaOAuth2ServerConfig>,
        resolver: Arc<dyn SaOAuth2DataResolver>,
        client_validator: Arc<dyn SaOAuth2ClientGrantValidator>,
    ) -> Self {
        Self {
            config,
            resolver,
            client_validator,
            user_authorize_client_check: Arc::new(|_, _| Ok(())),
            not_login_view: Arc::new(|| {
                Value::String("当前会话在 OAuth-Server 认证中心尚未登录".into())
            }),
            confirm_view: Arc::new(|_: &str, _: &[String]| {
                Value::String("本次操作需要用户授权".into())
            }),
            do_login_handle: Arc::new(
                |_: &str, _: &str| serde_json::json!({"code": 500, "msg": "login handler is not configured"}),
            ),
            scope_handlers: RwLock::new(BTreeMap::new()),
            grant_handlers: RwLock::new(BTreeMap::new()),
        }
    }

    pub fn with_user_authorize_client_check(
        mut self,
        check: Arc<UserAuthorizeClientCheck>,
    ) -> Self {
        self.user_authorize_client_check = check;
        self
    }

    pub fn with_server_functions(
        mut self,
        not_login_view: Arc<dyn SaOAuth2NotLoginViewFunction>,
        confirm_view: Arc<dyn SaOAuth2ConfirmViewFunction>,
        do_login_handle: Arc<dyn SaOAuth2DoLoginHandleFunction>,
    ) -> Self {
        self.not_login_view = not_login_view;
        self.confirm_view = confirm_view;
        self.do_login_handle = do_login_handle;
        self
    }

    pub fn not_login_view(&self) -> Value {
        self.not_login_view.get()
    }

    pub fn confirm_view(&self, client_id: &str, scopes: &[String]) -> Value {
        self.confirm_view.apply(client_id, scopes)
    }

    pub fn do_login(&self, name: &str, password: &str) -> Value {
        self.do_login_handle.apply(name, password)
    }

    pub fn check_user_authorize_client(&self, login_id: &Value, client_id: &str) -> SaResult<()> {
        (self.user_authorize_client_check)(login_id, client_id)
    }

    fn lock_error(area: &str) -> SaTokenException {
        SaTokenException::with_code(-1, format!("OAuth2 {area} registry lock poisoned"))
    }

    pub fn register_scope_handler(
        &self,
        handler: Arc<dyn SaOAuth2ScopeHandlerInterface>,
    ) -> SaResult<()> {
        let scope = handler.handler_scope().to_owned();
        self.scope_handlers
            .write()
            .map_err(|_| Self::lock_error("scope"))?
            .insert(scope, handler);
        Ok(())
    }

    pub fn remove_scope_handler(
        &self,
        scope: &str,
    ) -> SaResult<Option<Arc<dyn SaOAuth2ScopeHandlerInterface>>> {
        Ok(self
            .scope_handlers
            .write()
            .map_err(|_| Self::lock_error("scope"))?
            .remove(scope))
    }

    pub fn register_grant_type_handler(
        &self,
        handler: Arc<dyn SaOAuth2GrantTypeHandlerInterface>,
    ) -> SaResult<()> {
        let grant_type = handler.handler_grant_type().to_owned();
        self.grant_handlers
            .write()
            .map_err(|_| Self::lock_error("grant_type"))?
            .insert(grant_type, handler);
        Ok(())
    }

    pub fn remove_grant_type_handler(
        &self,
        grant_type: &str,
    ) -> SaResult<Option<Arc<dyn SaOAuth2GrantTypeHandlerInterface>>> {
        Ok(self
            .grant_handlers
            .write()
            .map_err(|_| Self::lock_error("grant_type"))?
            .remove(grant_type))
    }

    fn parse_scopes(value: Option<&str>) -> Vec<String> {
        value
            .unwrap_or_default()
            .replace("%20", ",")
            .replace([' ', '+'], ",")
            .split(',')
            .map(str::trim)
            .filter(|scope| !scope.is_empty())
            .map(str::to_owned)
            .collect()
    }

    fn scope_handlers(
        &self,
        scopes: Option<&[String]>,
        refresh_only: bool,
    ) -> SaResult<Vec<Arc<dyn SaOAuth2ScopeHandlerInterface>>> {
        let registry = self
            .scope_handlers
            .read()
            .map_err(|_| Self::lock_error("scope"))?;
        let mut handlers = scopes
            .unwrap_or_default()
            .iter()
            .filter_map(|scope| registry.get(scope))
            .filter(|handler| !refresh_only || handler.refresh_access_token_is_work())
            .cloned()
            .collect::<Vec<_>>();
        if let Some(handler) = registry.get(SaOAuth2Consts::FINALLY_WORK_SCOPE)
            && (!refresh_only || handler.refresh_access_token_is_work())
        {
            handlers.push(Arc::clone(handler));
        }
        Ok(handlers)
    }

    fn work_access_token(
        &self,
        access_token: &mut AccessTokenModel,
        refresh_only: bool,
    ) -> SaResult<()> {
        let handlers = self.scope_handlers(access_token.scopes.as_deref(), refresh_only)?;
        for handler in handlers {
            handler
                .work_access_token(access_token)
                .map_err(|error| SaTokenException::with_code(error.code, error.message))?;
        }
        Ok(())
    }

    pub fn work_client_token(&self, client_token: &mut ClientTokenModel) -> SaResult<()> {
        let handlers = self.scope_handlers(client_token.scopes.as_deref(), false)?;
        for handler in handlers {
            handler
                .work_client_token(client_token)
                .map_err(|error| SaTokenException::with_code(error.code, error.message))?;
        }
        Ok(())
    }

    /// Validates and dispatches a token request to its registered grant handler.
    ///
    /// # Errors
    ///
    /// Returns 30126 for missing, unknown, or globally disabled grants; 30141 when the client did
    /// not enable the grant; otherwise propagates resolver, validator, and handler errors.
    pub async fn grant_type_auth(&self, request: &SaOAuth2Request) -> SaResult<AccessTokenModel> {
        let grant_type = request
            .param(SaOAuth2Param::GRANT_TYPE)
            .ok_or_else(|| SaTokenException::with_code(30126, "grant_type 不可为空"))?;
        if (grant_type == GrantType::AUTHORIZATION_CODE && !self.config.enable_authorization_code)
            || (grant_type == GrantType::PASSWORD && !self.config.enable_password)
        {
            return Err(SaTokenException::with_code(
                30126,
                format!("系统未开放的 grant_type: {grant_type}"),
            ));
        }
        let handler = self
            .grant_handlers
            .read()
            .map_err(|_| Self::lock_error("grant_type"))?
            .get(grant_type)
            .cloned()
            .ok_or_else(|| {
                SaTokenException::with_code(30126, format!("无效 grant_type: {grant_type}"))
            })?;
        let credentials = self.resolver.read_client_id_and_secret(request)?;
        let client_id = credentials.client_id.as_deref().unwrap_or_default();
        let client_secret = credentials.client_secret.as_deref().unwrap_or_default();
        let scopes = Self::parse_scopes(request.param(SaOAuth2Param::SCOPE));
        let client = self
            .client_validator
            .validate(client_id, client_secret, &scopes)?;
        if !client
            .allow_grant_types
            .iter()
            .any(|allowed| allowed == grant_type)
        {
            return Err(SaTokenException::with_code(
                30141,
                format!("应用未开放的 grant_type: {grant_type}"),
            ));
        }
        handler.get_access_token(request, client_id, &scopes).await
    }
}

impl SaOAuth2GenerateHooks for SaOAuth2Strategy {
    fn user_authorize_client_check(&self, login_id: &Value, client_id: &str) -> SaResult<()> {
        (self.user_authorize_client_check)(login_id, client_id)
    }

    fn work_access_token_by_scope(&self, access_token: &mut AccessTokenModel) -> SaResult<()> {
        self.work_access_token(access_token, false)
    }

    fn refresh_access_token_work_by_scope(
        &self,
        access_token: &mut AccessTokenModel,
    ) -> SaResult<()> {
        self.work_access_token(access_token, true)
    }

    fn work_client_token_by_scope(&self, client_token: &mut ClientTokenModel) -> SaResult<()> {
        self.work_client_token(client_token)
    }
}
