//! Asynchronous login and authorization logic backed by an isolated runtime.

use std::sync::Arc;

use crate::exception::{SaResult, SaTokenException};
use crate::runtime::AsyncSaTokenRuntime;
use crate::session::sa_session::SaSession;
use crate::session::sa_terminal_info::SaTerminalInfo;
use crate::stp::parameter::sa_login_parameter::SaLoginParameter;
use crate::stp::sa_token_info::SaTokenInfo;

use super::shared;

/// Async counterpart of [`super::stp_logic::StpLogic`].
///
/// This type never reads the process-global [`crate::sa_manager::SaManager`].
/// Each instance owns an explicit runtime, which makes concurrent tests and
/// multi-tenant applications independent.
pub struct AsyncStpLogic {
    login_type: String,
    runtime: Arc<AsyncSaTokenRuntime>,
}

impl AsyncStpLogic {
    /// Creates an async logic instance for a login type and isolated runtime.
    pub fn new(login_type: impl Into<String>, runtime: Arc<AsyncSaTokenRuntime>) -> Self {
        Self {
            login_type: login_type.into(),
            runtime,
        }
    }

    /// Returns the account type handled by this instance.
    pub fn login_type(&self) -> &str {
        &self.login_type
    }

    /// Returns the owned runtime.
    pub fn runtime(&self) -> &Arc<AsyncSaTokenRuntime> {
        &self.runtime
    }

    /// Creates a login using default parameters.
    pub async fn login(&self, login_id: &str) -> SaResult<String> {
        self.login_with_param(login_id, &SaLoginParameter::default())
            .await
    }

    /// Creates a login using explicit device and timeout parameters.
    pub async fn login_with_param(
        &self,
        login_id: &str,
        parameter: &SaLoginParameter,
    ) -> SaResult<String> {
        if !parameter.get_is_concurrent(self.runtime.config()) {
            self.replaced_by_login_id(login_id).await?;
        }

        let session_key = self.session_key(login_id);
        let existing_session = self.runtime.dao().get_session(&session_key).await?;
        let session_existed = existing_session.is_some();
        let mut session = existing_session.unwrap_or_else(|| {
            let mut session = SaSession::new(&session_key);
            session.set_login_type(&self.login_type);
            session.set_login_id(login_id);
            session.set_session_type("login");
            session
        });

        let token_value = if let Some(token) = parameter.token.as_deref() {
            token.to_string()
        } else if parameter.get_is_concurrent(self.runtime.config())
            && parameter.get_is_share(self.runtime.config())
        {
            let candidates = session
                .terminal_list()
                .iter()
                .filter(|terminal| terminal.device_type() == parameter.device_type)
                .map(|terminal| terminal.token_value().to_string())
                .collect::<Vec<_>>();
            let mut reusable = None;
            for candidate in candidates {
                if self.get_login_id_by_token(&candidate).await?.is_some() {
                    reusable = Some(candidate);
                    break;
                }
            }
            reusable.unwrap_or_else(|| shared::create_token_value(self.runtime.config()))
        } else {
            shared::create_token_value(self.runtime.config())
        };

        let mut terminal = SaTerminalInfo::new(
            session.terminal_list().len() as i32 + 1,
            &token_value,
            &parameter.device_type,
        );
        terminal.set_device_id(&parameter.device_id);
        if let Some(extra) = &parameter.terminal_extra_data {
            terminal.set_extra("extra", extra.clone());
        }
        session.add_terminal(terminal);

        let max_login_count = parameter.get_max_login_count(self.runtime.config());
        let overflow_tokens =
            if max_login_count >= 0 && session.terminal_list().len() > max_login_count as usize {
                session
                    .terminal_list()
                    .iter()
                    .take(session.terminal_list().len() - max_login_count as usize)
                    .map(|terminal| terminal.token_value().to_string())
                    .collect::<Vec<_>>()
            } else {
                Vec::new()
            };
        for overflow_token in &overflow_tokens {
            session.remove_terminal(overflow_token);
            self.runtime
                .dao()
                .delete(&self.token_key(overflow_token))
                .await?;
            self.runtime
                .dao()
                .delete(&self.last_active_key(overflow_token))
                .await?;
            for listener in self.runtime.listeners() {
                listener.do_logout(&self.login_type, login_id, overflow_token);
            }
        }

        let timeout = parameter.get_timeout(self.runtime.config());
        if session_existed {
            self.runtime.dao().update_session(&session).await?;
        } else {
            self.runtime.dao().set_session(&session, timeout).await?;
        }
        self.runtime
            .dao()
            .set(&self.token_key(&token_value), login_id, timeout)
            .await?;
        if self.runtime.config().right_now_create_token_session {
            self.get_token_session_by_token(&token_value, true).await?;
        }
        self.set_last_active_to_now_with(&token_value, parameter.active_timeout, timeout)
            .await?;
        self.write_token_to_context(&token_value);

        for listener in self.runtime.listeners() {
            listener.do_login(&self.login_type, login_id, &token_value, parameter);
        }
        Ok(token_value)
    }

    /// Logs out the token attached to the current request context.
    pub async fn logout(&self) -> SaResult<()> {
        if let Some(token_value) = self.get_token_value() {
            self.logout_by_token_value(&token_value).await?;
        }
        Ok(())
    }

    /// Logs out every terminal owned by an account.
    pub async fn logout_by_login_id(&self, login_id: &str) -> SaResult<()> {
        for token in self.get_token_value_list_by_login_id(login_id).await? {
            self.logout_by_token_value(&token).await?;
        }
        self.runtime
            .dao()
            .delete_session(&self.session_key(login_id))
            .await
    }

    /// Logs out one token and removes it from its login session.
    pub async fn logout_by_token_value(&self, token_value: &str) -> SaResult<()> {
        let Some(login_id) = self.get_login_id_by_token(token_value).await? else {
            return Ok(());
        };
        self.delete_token_state(token_value, &login_id).await?;
        for listener in self.runtime.listeners() {
            listener.do_logout(&self.login_type, &login_id, token_value);
        }
        Ok(())
    }

    /// Kicks every token for an account offline.
    pub async fn kickout_by_login_id(&self, login_id: &str) -> SaResult<()> {
        for token in self.get_token_value_list_by_login_id(login_id).await? {
            self.kickout_by_token_value(&token).await?;
        }
        Ok(())
    }

    /// Kicks one token offline.
    pub async fn kickout_by_token_value(&self, token_value: &str) -> SaResult<()> {
        let Some(login_id) = self.get_login_id_by_token(token_value).await? else {
            return Ok(());
        };
        self.delete_token_state(token_value, &login_id).await?;
        for listener in self.runtime.listeners() {
            listener.do_kickout(&self.login_type, &login_id, token_value);
        }
        Ok(())
    }

    /// Replaces every token for an account.
    pub async fn replaced_by_login_id(&self, login_id: &str) -> SaResult<()> {
        for token in self.get_token_value_list_by_login_id(login_id).await? {
            self.replaced_by_token_value(&token).await?;
        }
        Ok(())
    }

    /// Replaces one token and removes its active terminal from the login session.
    pub async fn replaced_by_token_value(&self, token_value: &str) -> SaResult<()> {
        let Some(login_id) = self.get_login_id_by_token(token_value).await? else {
            return Ok(());
        };
        for listener in self.runtime.listeners() {
            listener.do_replaced(&self.login_type, &login_id, token_value);
        }
        self.delete_token_state(token_value, &login_id).await
    }

    /// Returns whether the current context carries a valid login token.
    pub async fn is_login(&self) -> SaResult<bool> {
        Ok(self.get_login_id_default_null().await?.is_some())
    }

    /// Fails when the current context has no valid login.
    pub async fn check_login(&self) -> SaResult<()> {
        if self.is_login().await? {
            Ok(())
        } else {
            Err(SaTokenException::not_login("未登录", &self.login_type))
        }
    }

    /// Returns the current login id or a not-login error.
    pub async fn get_login_id(&self) -> SaResult<String> {
        self.get_login_id_default_null()
            .await?
            .ok_or_else(|| SaTokenException::not_login("未登录", &self.login_type))
    }

    /// Returns the current login id, preserving the missing/error distinction.
    pub async fn get_login_id_default_null(&self) -> SaResult<Option<String>> {
        let Some(token_value) = self.get_token_value() else {
            return Ok(None);
        };
        self.get_login_id_by_token(&token_value).await
    }

    /// Resolves a login id from a token.
    pub async fn get_login_id_by_token(&self, token_value: &str) -> SaResult<Option<String>> {
        self.runtime.dao().get(&self.token_key(token_value)).await
    }

    /// Reads the token from header, cookie, parameter, then request storage.
    pub fn get_token_value(&self) -> Option<String> {
        let config = self.runtime.config();
        let context = self.runtime.context();
        if context.is_valid() {
            let request = context.request();
            let token = if config.is_read_header {
                request.get_header(&config.token_name)
            } else {
                None
            }
            .or_else(|| {
                config
                    .is_read_cookie
                    .then(|| request.get_cookie_value(&config.token_name))
                    .flatten()
            })
            .or_else(|| {
                config
                    .is_read_body
                    .then(|| request.get_param(&config.token_name))
                    .flatten()
            });
            if let Some(token) = token.filter(|value| !value.is_empty()) {
                return Some(self.cut_token_prefix(&token));
            }
            return context
                .storage()
                .get(&config.token_name)
                .filter(|value| !value.is_empty());
        }
        None
    }

    /// Returns stable metadata for the token in the current request context.
    pub async fn get_token_info(&self) -> SaResult<SaTokenInfo> {
        let token_value = self
            .get_token_value()
            .ok_or_else(|| SaTokenException::not_login("未提供 Token", &self.login_type))?;
        let login_id = self
            .get_login_id_by_token(&token_value)
            .await?
            .ok_or_else(|| SaTokenException::not_login("Token 无效", &self.login_type))?;
        let mut info = SaTokenInfo::new(&self.runtime.config().token_name, &token_value);
        info.login_id = login_id;
        info.token_timeout = self.get_token_timeout_by_token(&token_value).await?;
        info.login_device_type = self
            .get_login_device_type_by_token(&token_value)
            .await?
            .unwrap_or_default();
        Ok(info)
    }

    /// Returns the login session for an account.
    pub async fn get_session_by_login_id(&self, login_id: &str) -> SaResult<SaSession> {
        self.runtime
            .dao()
            .get_session(&self.session_key(login_id))
            .await?
            .ok_or_else(|| SaTokenException::other("Session 不存在"))
    }

    /// Returns the current token session, creating it lazily when absent.
    pub async fn get_token_session(&self) -> SaResult<SaSession> {
        let token_value = self
            .get_token_value()
            .ok_or_else(|| SaTokenException::other("Token-Session 获取失败：token 为空"))?;
        self.get_token_session_by_token(&token_value, true)
            .await?
            .ok_or_else(|| SaTokenException::other("Token-Session 不存在"))
    }

    /// Returns a token session and optionally creates it for a valid login token.
    pub async fn get_token_session_by_token(
        &self,
        token_value: &str,
        is_create: bool,
    ) -> SaResult<Option<SaSession>> {
        if token_value.is_empty() {
            return Err(SaTokenException::other(
                "Token-Session 获取失败：token 为空",
            ));
        }
        let session_key = self.token_session_key(token_value);
        if let Some(session) = self.runtime.dao().get_session(&session_key).await? {
            return Ok(Some(session));
        }
        if !is_create {
            return Ok(None);
        }
        if self.get_login_id_by_token(token_value).await?.is_none() {
            return Err(SaTokenException::other(format!(
                "Token-Session 获取失败，token 无效: {token_value}"
            )));
        }
        let mut session = SaSession::new(&session_key);
        session.set_session_type("token");
        session.set_login_type(&self.login_type);
        session.set_token(token_value);
        let timeout = self.get_token_timeout_by_token(token_value).await?;
        self.runtime.dao().set_session(&session, timeout).await?;
        Ok(Some(session))
    }

    /// Returns the current anonymous token session.
    pub async fn get_anon_token_session(&self) -> SaResult<SaSession> {
        self.get_anon_token_session_create(true)
            .await?
            .ok_or_else(|| SaTokenException::other("Anon Token-Session 不存在"))
    }

    /// Returns the current anonymous token session, optionally creating it.
    pub async fn get_anon_token_session_create(
        &self,
        is_create: bool,
    ) -> SaResult<Option<SaSession>> {
        if let Some(token_value) = self.get_token_value() {
            if !token_value.is_empty() {
                if let Some(session) = self.get_token_session_by_token(&token_value, false).await? {
                    return Ok(Some(session));
                }
                if self.get_login_id_by_token(&token_value).await?.is_some() {
                    return self
                        .get_token_session_by_token(&token_value, is_create)
                        .await;
                }
            }
        }

        if !is_create {
            return Ok(None);
        }

        let token_value = shared::create_token_value(self.runtime.config());
        self.set_last_active_to_now(&token_value).await?;
        self.write_token_to_context(&token_value);

        let session_key = self.token_session_key(&token_value);
        let mut session = SaSession::new(&session_key);
        session.set_session_type(crate::util::sa_token_consts::SESSION_TYPE_ANON);
        session.set_login_type(&self.login_type);
        session.set_token(&token_value);
        let timeout = self.runtime.config().timeout;
        self.runtime.dao().set_session(&session, timeout).await?;
        Ok(Some(session))
    }

    /// Returns all token values attached to an account session.
    pub async fn get_token_value_list_by_login_id(&self, login_id: &str) -> SaResult<Vec<String>> {
        let Some(session) = self
            .runtime
            .dao()
            .get_session(&self.session_key(login_id))
            .await?
        else {
            return Ok(Vec::new());
        };
        Ok(session
            .terminal_list()
            .iter()
            .map(|terminal| terminal.token_value().to_string())
            .collect())
    }

    /// Returns a token's remaining TTL.
    pub async fn get_token_timeout_by_token(&self, token_value: &str) -> SaResult<i64> {
        self.runtime
            .dao()
            .get_timeout(&self.token_key(token_value))
            .await
    }

    /// Returns the current token's remaining TTL, or `-2` when no token is provided.
    pub async fn get_token_timeout(&self) -> SaResult<i64> {
        match self.get_token_value() {
            Some(token_value) => self.get_token_timeout_by_token(&token_value).await,
            None => Ok(-2),
        }
    }

    /// Changes the current token's TTL.
    pub async fn renew_timeout(&self, timeout: i64) -> SaResult<()> {
        let token_value = self
            .get_token_value()
            .ok_or_else(|| SaTokenException::not_login("未提供 Token", &self.login_type))?;
        self.renew_timeout_by_token(&token_value, timeout).await
    }

    /// Changes a token's TTL and refreshes its activity marker.
    pub async fn renew_timeout_by_token(&self, token_value: &str, timeout: i64) -> SaResult<()> {
        self.runtime
            .dao()
            .update_timeout(&self.token_key(token_value), timeout)
            .await?;
        self.set_last_active_to_now(token_value).await
    }

    /// Returns the remaining active timeout for the current token.
    pub async fn get_token_active_timeout(&self) -> SaResult<i64> {
        let Some(token_value) = self.get_token_value() else {
            return Ok(-2);
        };
        self.get_token_active_timeout_by_token(&token_value).await
    }

    /// Returns a token's remaining active timeout (`-1` unlimited, `-2` frozen/missing).
    pub async fn get_token_active_timeout_by_token(&self, token_value: &str) -> SaResult<i64> {
        let configured = self.runtime.config().active_timeout;
        if configured == -1 {
            return Ok(-1);
        }
        let Some(value) = self
            .runtime
            .dao()
            .get(&self.last_active_key(token_value))
            .await?
        else {
            return Ok(-2);
        };
        let mut parts = value.splitn(2, ',');
        let last_active = parts
            .next()
            .and_then(|value| value.parse::<i64>().ok())
            .ok_or_else(|| SaTokenException::other("最后活跃时间格式无效"))?;
        let allowed = parts
            .next()
            .and_then(|value| value.parse::<i64>().ok())
            .unwrap_or(configured);
        if allowed == -1 {
            return Ok(-1);
        }
        let elapsed = (now_millis().saturating_sub(last_active)) / 1_000;
        let remaining = allowed - elapsed;
        Ok(if remaining < 0 { -2 } else { remaining })
    }

    /// Fails when the current token exceeded its active timeout.
    pub async fn check_active_timeout(&self) -> SaResult<()> {
        if self.get_token_active_timeout().await? == -2 {
            Err(SaTokenException::not_login(
                "Token 已被冻结",
                &self.login_type,
            ))
        } else {
            Ok(())
        }
    }

    /// Refreshes the current token's active timestamp.
    pub async fn update_last_active_to_now(&self) -> SaResult<()> {
        let token_value = self
            .get_token_value()
            .ok_or_else(|| SaTokenException::not_login("未提供 Token", &self.login_type))?;
        self.set_last_active_to_now(&token_value).await
    }

    /// Returns the roles supplied by this runtime's authorization provider.
    pub async fn get_role_list(&self) -> SaResult<Vec<String>> {
        let login_id = self.get_login_id().await?;
        Ok(self
            .runtime
            .stp_interface()
            .get_role_list(&login_id, &self.login_type))
    }

    /// Returns whether the current account has a role.
    pub async fn has_role(&self, role: &str) -> SaResult<bool> {
        Ok(self.get_role_list().await?.iter().any(|item| item == role))
    }

    /// Fails when the current account lacks a role.
    pub async fn check_role(&self, role: &str) -> SaResult<()> {
        if self.has_role(role).await? {
            Ok(())
        } else {
            Err(SaTokenException::not_role(role, &self.login_type))
        }
    }

    /// Returns the permissions supplied by this runtime's authorization provider.
    pub async fn get_permission_list(&self) -> SaResult<Vec<String>> {
        let login_id = self.get_login_id().await?;
        Ok(self
            .runtime
            .stp_interface()
            .get_permission_list(&login_id, &self.login_type))
    }

    /// Checks a permission against the runtime-local provider.
    pub async fn has_permission(&self, permission: &str) -> SaResult<bool> {
        Ok(self
            .get_permission_list()
            .await?
            .iter()
            .any(|item| item == permission))
    }

    /// Fails when the current account lacks a permission.
    pub async fn check_permission(&self, permission: &str) -> SaResult<()> {
        if self.has_permission(permission).await? {
            Ok(())
        } else {
            Err(SaTokenException::not_permission(
                permission,
                &self.login_type,
            ))
        }
    }

    /// Returns the current terminal's device type.
    pub async fn get_login_device_type(&self) -> SaResult<String> {
        let token_value = self
            .get_token_value()
            .ok_or_else(|| SaTokenException::not_login("未提供 Token", &self.login_type))?;
        self.get_login_device_type_by_token(&token_value)
            .await?
            .ok_or_else(|| SaTokenException::other("无法获取设备类型"))
    }

    /// Returns a token's device type when the terminal exists.
    pub async fn get_login_device_type_by_token(
        &self,
        token_value: &str,
    ) -> SaResult<Option<String>> {
        Ok(self
            .terminal_by_token(token_value)
            .await?
            .map(|terminal| terminal.device_type().to_string()))
    }

    /// Returns the current terminal's device id.
    pub async fn get_login_device_id(&self) -> SaResult<String> {
        let token_value = self
            .get_token_value()
            .ok_or_else(|| SaTokenException::not_login("未提供 Token", &self.login_type))?;
        self.get_login_device_id_by_token(&token_value)
            .await?
            .ok_or_else(|| SaTokenException::other("无法获取设备 ID"))
    }

    /// Returns a token's device id when the terminal exists.
    pub async fn get_login_device_id_by_token(
        &self,
        token_value: &str,
    ) -> SaResult<Option<String>> {
        Ok(self
            .terminal_by_token(token_value)
            .await?
            .map(|terminal| terminal.device_id().to_string()))
    }

    /// Returns every terminal for an account.
    pub async fn get_terminal_list_by_login_id(
        &self,
        login_id: &str,
    ) -> SaResult<Vec<SaTerminalInfo>> {
        Ok(self
            .get_session_by_login_id(login_id)
            .await?
            .terminal_list()
            .to_vec())
    }

    /// Returns the terminal represented by a token.
    pub async fn get_terminal_info_by_token(&self, token_value: &str) -> SaResult<SaTerminalInfo> {
        self.terminal_by_token(token_value)
            .await?
            .ok_or_else(|| SaTokenException::other("终端不存在"))
    }

    /// Disables the default `login` service for an account.
    pub async fn disable(&self, login_id: &str, time: i64) -> SaResult<()> {
        self.disable_level(login_id, crate::util::sa_token_consts::DEFAULT_DISABLE_LEVEL, time)
            .await
    }

    /// Disables a named service for an account.
    pub async fn disable_with_service(
        &self,
        login_id: &str,
        service: &str,
        time: i64,
    ) -> SaResult<()> {
        self.disable_level_with_service(
            login_id,
            service,
            crate::util::sa_token_consts::DEFAULT_DISABLE_LEVEL,
            time,
        )
        .await
    }

    /// Disables an account at a specific level.
    pub async fn disable_level(&self, login_id: &str, level: i32, time: i64) -> SaResult<()> {
        self.disable_level_with_service(
            login_id,
            crate::util::sa_token_consts::DEFAULT_DISABLE_SERVICE,
            level,
            time,
        )
        .await
    }

    /// Disables a named service at a specific level.
    pub async fn disable_level_with_service(
        &self,
        login_id: &str,
        service: &str,
        level: i32,
        time: i64,
    ) -> SaResult<()> {
        self.validate_disable_args(login_id, service, level)?;
        self.runtime
            .dao()
            .set(
                &self.disable_key(login_id, service),
                &level.to_string(),
                time,
            )
            .await?;
        for listener in self.runtime.listeners() {
            listener.do_disable(&self.login_type, login_id, service, level, time);
        }
        Ok(())
    }

    /// Removes the default service disable marker.
    pub async fn untie_disable(&self, login_id: &str) -> SaResult<()> {
        self.untie_disable_with_service(login_id, crate::util::sa_token_consts::DEFAULT_DISABLE_SERVICE)
            .await
    }

    /// Removes a named service disable marker.
    pub async fn untie_disable_with_service(&self, login_id: &str, service: &str) -> SaResult<()> {
        if login_id.is_empty() {
            return Err(SaTokenException::with_code(
                crate::error::SaErrorCode::CODE_11062,
                "请提供要解禁的账号",
            ));
        }
        if service.is_empty() {
            return Err(SaTokenException::with_code(
                crate::error::SaErrorCode::CODE_11063,
                "请提供要解禁的服务",
            ));
        }
        self.runtime
            .dao()
            .delete(&self.disable_key(login_id, service))
            .await?;
        for listener in self.runtime.listeners() {
            listener.do_untie_disable(&self.login_type, login_id, service);
        }
        Ok(())
    }

    /// Returns the disable level for the default service.
    pub async fn get_disable_level(&self, login_id: &str) -> SaResult<i32> {
        self.get_disable_level_with_service(
            login_id,
            crate::util::sa_token_consts::DEFAULT_DISABLE_SERVICE,
        )
        .await
    }

    /// Returns the disable level for a named service.
    pub async fn get_disable_level_with_service(
        &self,
        login_id: &str,
        service: &str,
    ) -> SaResult<i32> {
        if let Some(value) = self
            .runtime
            .dao()
            .get(&self.disable_key(login_id, service))
            .await?
        {
            return Ok(value.parse().unwrap_or(crate::util::sa_token_consts::DEFAULT_DISABLE_LEVEL));
        }

        let wrapper = self.runtime.stp_interface().is_disabled(login_id, service);
        if wrapper.disable_time == crate::util::sa_token_consts::NEVER_EXPIRE
            || wrapper.disable_time > 0
        {
            self.disable_level_with_service(
                login_id,
                service,
                wrapper.disable_level,
                wrapper.disable_time,
            )
            .await?;
        }
        Ok(wrapper.disable_level)
    }

    /// Returns whether the default service is disabled.
    pub async fn is_disable(&self, login_id: &str) -> SaResult<bool> {
        self.is_disable_level_with_service(
            login_id,
            crate::util::sa_token_consts::DEFAULT_DISABLE_SERVICE,
            crate::util::sa_token_consts::MIN_DISABLE_LEVEL,
        )
        .await
    }

    /// Returns whether a service is disabled to the given level.
    pub async fn is_disable_level(&self, login_id: &str, level: i32) -> SaResult<bool> {
        self.is_disable_level_with_service(
            login_id,
            crate::util::sa_token_consts::DEFAULT_DISABLE_SERVICE,
            level,
        )
        .await
    }

    /// Returns whether a named service is disabled to the given level.
    pub async fn is_disable_level_with_service(
        &self,
        login_id: &str,
        service: &str,
        level: i32,
    ) -> SaResult<bool> {
        let disable_level = self.get_disable_level_with_service(login_id, service).await?;
        if disable_level == crate::util::sa_token_consts::NOT_DISABLE_LEVEL {
            return Ok(false);
        }
        Ok(disable_level >= level)
    }

    /// Fails when the default service is disabled.
    pub async fn check_disable(&self, login_id: &str) -> SaResult<()> {
        self.check_disable_level_with_service(
            login_id,
            crate::util::sa_token_consts::DEFAULT_DISABLE_SERVICE,
            crate::util::sa_token_consts::MIN_DISABLE_LEVEL,
        )
        .await
    }

    /// Fails when a service is disabled to the given level.
    pub async fn check_disable_level(&self, login_id: &str, level: i32) -> SaResult<()> {
        self.check_disable_level_with_service(
            login_id,
            crate::util::sa_token_consts::DEFAULT_DISABLE_SERVICE,
            level,
        )
        .await
    }

    /// Fails when a named service is disabled to the given level.
    pub async fn check_disable_level_with_service(
        &self,
        login_id: &str,
        service: &str,
        level: i32,
    ) -> SaResult<()> {
        let disable_level = self.get_disable_level_with_service(login_id, service).await?;
        if disable_level == crate::util::sa_token_consts::NOT_DISABLE_LEVEL {
            return Ok(());
        }
        if disable_level >= level {
            let disable_time = self
                .runtime
                .dao()
                .get_timeout(&self.disable_key(login_id, service))
                .await?;
            return Err(SaTokenException::disable_service_level(
                login_id,
                service,
                disable_level,
                level,
                disable_time,
            ));
        }
        Ok(())
    }

    /// Opens the default safe-auth marker for the current token.
    pub async fn open_safe(&self, safe_time: i64) -> SaResult<()> {
        self.open_safe_with_service("", safe_time).await
    }

    /// Opens a named safe-auth marker for the current token.
    pub async fn open_safe_with_service(&self, service: &str, safe_time: i64) -> SaResult<()> {
        let token_value = self
            .get_token_value()
            .ok_or_else(|| SaTokenException::not_login("未提供 Token", &self.login_type))?;
        self.runtime
            .dao()
            .set(
                &self.safe_key(&token_value, service),
                "SAFE_AUTH_SAVE_VALUE",
                safe_time,
            )
            .await?;
        for listener in self.runtime.listeners() {
            listener.do_open_safe(&self.login_type, &token_value, service, safe_time);
        }
        Ok(())
    }

    /// Returns whether the default safe-auth marker is active.
    pub async fn is_safe(&self) -> SaResult<bool> {
        self.is_safe_with_service("").await
    }

    /// Returns whether a named safe-auth marker is active.
    pub async fn is_safe_with_service(&self, service: &str) -> SaResult<bool> {
        let Some(token_value) = self.get_token_value() else {
            return Ok(false);
        };
        Ok(self
            .runtime
            .dao()
            .get(&self.safe_key(&token_value, service))
            .await?
            .is_some())
    }

    /// Fails when the default safe-auth marker is absent.
    pub async fn check_safe(&self) -> SaResult<()> {
        if self.is_safe().await? {
            Ok(())
        } else {
            Err(SaTokenException::not_safe("", &self.login_type))
        }
    }

    /// Closes the default safe-auth marker.
    pub async fn close_safe(&self) -> SaResult<()> {
        self.close_safe_with_service("").await
    }

    /// Closes a named safe-auth marker.
    pub async fn close_safe_with_service(&self, service: &str) -> SaResult<()> {
        if let Some(token_value) = self.get_token_value() {
            self.runtime
                .dao()
                .delete(&self.safe_key(&token_value, service))
                .await?;
            for listener in self.runtime.listeners() {
                listener.do_close_safe(&self.login_type, &token_value, service);
            }
        }
        Ok(())
    }

    /// Switches the request-local identity to another login id.
    pub async fn switch_to(&self, login_id: &str) -> SaResult<()> {
        self.check_login().await?;
        self.runtime
            .context()
            .storage()
            .set(&self.switch_key(), login_id);
        Ok(())
    }

    /// Ends request-local identity switching.
    pub fn end_switch(&self) {
        self.runtime.context().storage().delete(&self.switch_key());
    }

    /// Returns whether request-local identity switching is active.
    pub fn is_switch(&self) -> bool {
        self.runtime
            .context()
            .storage()
            .get(&self.switch_key())
            .is_some()
    }

    /// Returns the request-local switched login id.
    pub fn get_switch_login_id(&self) -> Option<String> {
        self.runtime.context().storage().get(&self.switch_key())
    }

    async fn terminal_by_token(&self, token_value: &str) -> SaResult<Option<SaTerminalInfo>> {
        let Some(login_id) = self.get_login_id_by_token(token_value).await? else {
            return Ok(None);
        };
        let Some(session) = self
            .runtime
            .dao()
            .get_session(&self.session_key(&login_id))
            .await?
        else {
            return Ok(None);
        };
        Ok(session.get_terminal(token_value).cloned())
    }

    async fn set_last_active_to_now(&self, token_value: &str) -> SaResult<()> {
        let timeout = self.get_token_timeout_by_token(token_value).await?;
        self.set_last_active_to_now_with(token_value, None, timeout)
            .await
    }

    async fn set_last_active_to_now_with(
        &self,
        token_value: &str,
        active_timeout: Option<i64>,
        timeout: i64,
    ) -> SaResult<()> {
        if self.runtime.config().active_timeout == -1 && active_timeout.unwrap_or(-1) == -1 {
            return Ok(());
        }
        let value = match active_timeout {
            Some(active_timeout) => format!("{},{active_timeout}", now_millis()),
            None => now_millis().to_string(),
        };
        self.runtime
            .dao()
            .set(&self.last_active_key(token_value), &value, timeout)
            .await
    }

    async fn delete_token_state(&self, token_value: &str, login_id: &str) -> SaResult<()> {
        self.runtime
            .dao()
            .delete(&self.token_key(token_value))
            .await?;
        self.runtime
            .dao()
            .delete(&self.last_active_key(token_value))
            .await?;
        self.runtime
            .dao()
            .delete_session(&self.token_session_key(token_value))
            .await?;
        let session_key = self.session_key(login_id);
        if let Some(mut session) = self.runtime.dao().get_session(&session_key).await? {
            session.remove_terminal(token_value);
            self.runtime.dao().update_session(&session).await?;
        }
        Ok(())
    }

    fn write_token_to_context(&self, token_value: &str) {
        let config = self.runtime.config();
        let context = self.runtime.context();
        if context.is_valid() {
            context.storage().set(&config.token_name, token_value);
            if config.is_read_cookie {
                context
                    .response()
                    .add_cookie(crate::context::model::sa_cookie::SaCookie::new(
                        &config.token_name,
                        token_value,
                    ));
            }
            if config.is_write_header {
                context
                    .response()
                    .set_header(&config.token_name, token_value);
            }
        }
    }

    fn token_key(&self, token_value: &str) -> String {
        shared::token_key(
            &self.runtime.config().token_name,
            &self.login_type,
            token_value,
        )
    }

    fn session_key(&self, login_id: &str) -> String {
        shared::session_key(
            &self.runtime.config().token_name,
            &self.login_type,
            login_id,
        )
    }

    fn token_session_key(&self, token_value: &str) -> String {
        shared::token_session_key(
            &self.runtime.config().token_name,
            &self.login_type,
            token_value,
        )
    }

    fn last_active_key(&self, token_value: &str) -> String {
        shared::last_active_key(
            &self.runtime.config().token_name,
            &self.login_type,
            token_value,
        )
    }

    fn disable_key(&self, login_id: &str, service: &str) -> String {
        shared::disable_key(
            &self.runtime.config().token_name,
            &self.login_type,
            login_id,
            service,
        )
    }

    fn safe_key(&self, token_value: &str, service: &str) -> String {
        shared::safe_key(
            &self.runtime.config().token_name,
            &self.login_type,
            token_value,
            service,
        )
    }

    fn switch_key(&self) -> String {
        shared::switch_key(&self.login_type)
    }

    fn validate_disable_args(&self, login_id: &str, service: &str, level: i32) -> SaResult<()> {
        if login_id.is_empty() {
            return Err(SaTokenException::with_code(
                crate::error::SaErrorCode::CODE_11062,
                "请提供要封禁的账号",
            ));
        }
        if service.is_empty() {
            return Err(SaTokenException::with_code(
                crate::error::SaErrorCode::CODE_11063,
                "请提供要封禁的服务",
            ));
        }
        if level < crate::util::sa_token_consts::MIN_DISABLE_LEVEL && level != 0 {
            return Err(SaTokenException::with_code(
                crate::error::SaErrorCode::CODE_11064,
                format!(
                    "封禁等级不可以小于最小值：{} (0除外)",
                    crate::util::sa_token_consts::MIN_DISABLE_LEVEL
                ),
            ));
        }
        Ok(())
    }

    fn cut_token_prefix(&self, token: &str) -> String {
        let prefix = &self.runtime.config().token_prefix;
        token.strip_prefix(prefix).unwrap_or(token).to_string()
    }
}

fn now_millis() -> i64 {
    use std::time::{SystemTime, UNIX_EPOCH};

    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |duration| duration.as_millis() as i64)
}
