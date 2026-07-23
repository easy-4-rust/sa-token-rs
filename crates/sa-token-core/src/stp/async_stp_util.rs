//! Instance facade for isolated asynchronous Sa-Token runtimes.

use std::sync::Arc;

use crate::exception::SaResult;
use crate::runtime::AsyncSaTokenRuntime;
use crate::session::sa_session::SaSession;
use crate::session::sa_terminal_info::SaTerminalInfo;
use crate::stp::parameter::sa_login_parameter::SaLoginParameter;
use crate::stp::sa_token_info::SaTokenInfo;

use super::async_stp_logic::AsyncStpLogic;

/// High-frequency async facade that never relies on global mutable state.
#[derive(Clone)]
pub struct AsyncStpUtil {
    logic: Arc<AsyncStpLogic>,
}

impl AsyncStpUtil {
    /// Builds a facade from an explicit runtime.
    pub fn new(login_type: impl Into<String>, runtime: Arc<AsyncSaTokenRuntime>) -> Self {
        Self {
            logic: Arc::new(AsyncStpLogic::new(login_type, runtime)),
        }
    }

    /// Returns the underlying async logic instance.
    pub fn logic(&self) -> &Arc<AsyncStpLogic> {
        &self.logic
    }

    /// Logs an account in and returns its token.
    pub async fn login(&self, login_id: &str) -> SaResult<String> {
        self.logic.login(login_id).await
    }

    /// Logs an account in with explicit parameters.
    pub async fn login_with_param(
        &self,
        login_id: &str,
        parameter: &SaLoginParameter,
    ) -> SaResult<String> {
        self.logic.login_with_param(login_id, parameter).await
    }

    /// Logs the current token out.
    pub async fn logout(&self) -> SaResult<()> {
        self.logic.logout().await
    }

    /// Logs all tokens for an account out.
    pub async fn logout_by_login_id(&self, login_id: &str) -> SaResult<()> {
        self.logic.logout_by_login_id(login_id).await
    }

    /// Kicks every token for an account offline.
    pub async fn kickout_by_login_id(&self, login_id: &str) -> SaResult<()> {
        self.logic.kickout_by_login_id(login_id).await
    }

    /// Replaces every token for an account.
    pub async fn replaced_by_login_id(&self, login_id: &str) -> SaResult<()> {
        self.logic.replaced_by_login_id(login_id).await
    }

    /// Returns whether the current context has a valid login.
    pub async fn is_login(&self) -> SaResult<bool> {
        self.logic.is_login().await
    }

    /// Returns the current login id.
    pub async fn get_login_id(&self) -> SaResult<String> {
        self.logic.get_login_id().await
    }

    /// Fails when the current context has no valid login.
    pub async fn check_login(&self) -> SaResult<()> {
        self.logic.check_login().await
    }

    /// Resolves a login id from a token.
    pub async fn get_login_id_by_token(&self, token: &str) -> SaResult<Option<String>> {
        self.logic.get_login_id_by_token(token).await
    }

    /// Returns an account's session.
    pub async fn get_session_by_login_id(&self, login_id: &str) -> SaResult<SaSession> {
        self.logic.get_session_by_login_id(login_id).await
    }

    /// Returns the current token session, creating it lazily when absent.
    pub async fn get_token_session(&self) -> SaResult<SaSession> {
        self.logic.get_token_session().await
    }

    /// Returns metadata for the current token.
    pub async fn get_token_info(&self) -> SaResult<SaTokenInfo> {
        self.logic.get_token_info().await
    }

    /// Returns the current token's remaining TTL.
    pub async fn get_token_timeout(&self) -> SaResult<i64> {
        self.logic.get_token_timeout().await
    }

    /// Changes the current token's TTL.
    pub async fn renew_timeout(&self, timeout: i64) -> SaResult<()> {
        self.logic.renew_timeout(timeout).await
    }

    /// Returns the current token's remaining active timeout.
    pub async fn get_token_active_timeout(&self) -> SaResult<i64> {
        self.logic.get_token_active_timeout().await
    }

    /// Fails when the current token exceeded its active timeout.
    pub async fn check_active_timeout(&self) -> SaResult<()> {
        self.logic.check_active_timeout().await
    }

    /// Refreshes the current token's active timestamp.
    pub async fn update_last_active_to_now(&self) -> SaResult<()> {
        self.logic.update_last_active_to_now().await
    }

    /// Returns an account's tokens.
    pub async fn get_token_value_list_by_login_id(&self, login_id: &str) -> SaResult<Vec<String>> {
        self.logic.get_token_value_list_by_login_id(login_id).await
    }

    /// Checks a permission using this runtime's provider.
    pub async fn has_permission(&self, permission: &str) -> SaResult<bool> {
        self.logic.has_permission(permission).await
    }

    /// Fails when the current account lacks a permission.
    pub async fn check_permission(&self, permission: &str) -> SaResult<()> {
        self.logic.check_permission(permission).await
    }

    /// Returns whether the current account has a role.
    pub async fn has_role(&self, role: &str) -> SaResult<bool> {
        self.logic.has_role(role).await
    }

    /// Fails when the current account lacks a role.
    pub async fn check_role(&self, role: &str) -> SaResult<()> {
        self.logic.check_role(role).await
    }

    /// Returns the current terminal's device type.
    pub async fn get_login_device_type(&self) -> SaResult<String> {
        self.logic.get_login_device_type().await
    }

    /// Returns the current terminal's device id.
    pub async fn get_login_device_id(&self) -> SaResult<String> {
        self.logic.get_login_device_id().await
    }

    /// Returns every terminal for an account.
    pub async fn get_terminal_list_by_login_id(
        &self,
        login_id: &str,
    ) -> SaResult<Vec<SaTerminalInfo>> {
        self.logic.get_terminal_list_by_login_id(login_id).await
    }

    /// Returns the current anonymous token session.
    pub async fn get_anon_token_session(&self) -> SaResult<SaSession> {
        self.logic.get_anon_token_session().await
    }

    /// Disables the default login service for an account.
    pub async fn disable(&self, login_id: &str, time: i64) -> SaResult<()> {
        self.logic.disable(login_id, time).await
    }

    /// Removes the default login service disable marker.
    pub async fn untie_disable(&self, login_id: &str) -> SaResult<()> {
        self.logic.untie_disable(login_id).await
    }

    /// Returns whether the default login service is disabled.
    pub async fn is_disable(&self, login_id: &str) -> SaResult<bool> {
        self.logic.is_disable(login_id).await
    }

    /// Fails when the default login service is disabled.
    pub async fn check_disable(&self, login_id: &str) -> SaResult<()> {
        self.logic.check_disable(login_id).await
    }

    /// Disables an account at a specific level.
    pub async fn disable_level(&self, login_id: &str, level: i32, time: i64) -> SaResult<()> {
        self.logic.disable_level(login_id, level, time).await
    }

    /// Returns the disable level for an account.
    pub async fn get_disable_level(&self, login_id: &str) -> SaResult<i32> {
        self.logic.get_disable_level(login_id).await
    }

    /// Returns whether an account is disabled to the given level.
    pub async fn is_disable_level(&self, login_id: &str, level: i32) -> SaResult<bool> {
        self.logic.is_disable_level(login_id, level).await
    }

    /// Fails when an account is disabled to the given level.
    pub async fn check_disable_level(&self, login_id: &str, level: i32) -> SaResult<()> {
        self.logic.check_disable_level(login_id, level).await
    }

    /// Opens the default safe-auth marker.
    pub async fn open_safe(&self, safe_time: i64) -> SaResult<()> {
        self.logic.open_safe(safe_time).await
    }

    /// Returns whether the default safe-auth marker is active.
    pub async fn is_safe(&self) -> SaResult<bool> {
        self.logic.is_safe().await
    }

    /// Closes the default safe-auth marker.
    pub async fn close_safe(&self) -> SaResult<()> {
        self.logic.close_safe().await
    }

    /// Switches the request-local identity to another account.
    pub async fn switch_to(&self, login_id: &str) -> SaResult<()> {
        self.logic.switch_to(login_id).await
    }

    /// Ends request-local identity switching.
    pub fn end_switch(&self) {
        self.logic.end_switch();
    }

    /// Returns whether request-local identity switching is active.
    pub fn is_switch(&self) -> bool {
        self.logic.is_switch()
    }

    /// Returns the request-local switched login id.
    pub fn get_switch_login_id(&self) -> Option<String> {
        self.logic.get_switch_login_id()
    }
}
