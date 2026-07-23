//! 核心逻辑（对应 Java `cn.dev33.satoken.stp.StpLogic`）。
use std::sync::Arc;

use crate::config::sa_token_config::SaTokenConfig;
use crate::exception::{SaResult, SaTokenException};
use crate::sa_manager::SaManager;
use crate::session::sa_session::SaSession;
use crate::session::sa_terminal_info::SaTerminalInfo;
use crate::stp::parameter::sa_login_parameter::SaLoginParameter;
use crate::stp::sa_token_info::SaTokenInfo;

/// Sa-Token 核心逻辑
///
/// 对应 Java `StpLogic`，封装了登录、登出、权限校验等核心逻辑。
pub struct StpLogic {
    /// 账号类型
    login_type: String,
}

impl StpLogic {
    /// 创建 StpLogic 实例
    pub fn new(login_type: impl Into<String>) -> Self {
        Self {
            login_type: login_type.into(),
        }
    }

    /// 获取账号类型
    pub fn login_type(&self) -> &str {
        &self.login_type
    }

    /// 获取全局配置
    pub fn config(&self) -> Arc<SaTokenConfig> {
        SaManager::config()
    }

    // ==================== 登录 ====================

    /// 登录
    pub fn login(&self, id: &str) -> SaResult<()> {
        self.login_with_param(id, &SaLoginParameter::default())
    }

    /// 登录（指定设备类型）
    pub fn login_with_device(&self, id: &str, device_type: &str) -> SaResult<()> {
        let param = SaLoginParameter::create().set_device_type(device_type);
        self.login_with_param(id, &param)
    }

    /// 登录（完整参数）
    pub fn login_with_param(&self, id: &str, param: &SaLoginParameter) -> SaResult<()> {
        // 创建登录会话
        let token_value = self.create_login_session(id, param)?;

        // 写入 Token 到当前请求
        self.set_token_value(&token_value)?;

        // 触发登录事件
        SaManager::listeners()
            .read()
            .unwrap()
            .iter()
            .for_each(|l| l.do_login(&self.login_type, id, &token_value, param));

        Ok(())
    }

    /// 创建登录会话
    pub fn create_login_session(&self, id: &str, param: &SaLoginParameter) -> SaResult<String> {
        if id.is_empty() {
            return Err(SaTokenException::with_code(
                crate::error::SaErrorCode::CODE_11002,
                "loginId 不能为空",
            ));
        }

        if !param.get_is_concurrent(&self.config()) {
            self.replaced_by_login_id(id)?;
        }

        let config = self.config();
        let dao = SaManager::sa_token_dao();
        let session_key = self.splicing_key_session(id);
        let existing_session = dao.get_session(&session_key)?;
        let session_existed = existing_session.is_some();
        let mut session = existing_session.unwrap_or_else(|| {
            let mut session = SaSession::new(&session_key);
            session.set_login_type(&self.login_type);
            session.set_login_id(id);
            session.set_session_type("login");
            session
        });

        let token_value = self.dist_usable_token(id, param, &session)?;

        let mut terminal = SaTerminalInfo::new(
            session.terminal_list().len() as i32 + 1,
            &token_value,
            &param.device_type,
        );
        terminal.set_device_id(&param.device_id);
        if let Some(ref extra) = param.terminal_extra_data {
            terminal.set_extra("extra", extra.clone());
        }
        session.add_terminal(terminal);

        let max_login_count = param.get_max_login_count(&config);
        let overflow_tokens = if max_login_count >= 0
            && session.terminal_list().len() > max_login_count as usize
        {
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
            dao.delete(&self.splicing_key_token_value(overflow_token))?;
            dao.delete(&self.splicing_key_last_active_time(overflow_token))?;
            SaManager::listeners()
                .read()
                .map_err(|_| SaTokenException::other("监听器读锁已损坏"))?
                .iter()
                .for_each(|listener| listener.do_logout(&self.login_type, id, overflow_token));
        }

        let timeout = param.get_timeout(&config);
        if session_existed {
            dao.update_session(&session)?;
        } else {
            dao.set_session(&session, timeout)?;
        }

        let token_key = self.splicing_key_token_value(&token_value);
        dao.set(&token_key, id, timeout)?;
        self.set_last_active_to_now(&token_value)?;

        if config.right_now_create_token_session {
            self.get_token_session_by_token_create(&token_value, true)?;
        }

        Ok(token_value)
    }

    /// 分配可用 Token（对应 Java `distUsableToken`）
    fn dist_usable_token(
        &self,
        id: &str,
        param: &SaLoginParameter,
        session: &SaSession,
    ) -> SaResult<String> {
        if let Some(token) = param.token.as_deref() {
            return Ok(token.to_string());
        }

        let config = self.config();
        if param.get_is_concurrent(&config) && param.get_is_share(&config) {
            for terminal in session
                .terminal_list()
                .iter()
                .filter(|terminal| terminal.device_type() == param.device_type)
            {
                let candidate = terminal.token_value();
                if self.get_login_id_by_token(candidate)?.is_some() {
                    return Ok(candidate.to_string());
                }
            }
        }

        Ok(self.create_token_value(id, param))
    }

    /// 获取或创建登录会话
    pub fn get_or_create_login_session(&self, id: &str) -> SaResult<SaSession> {
        let session_key = self.splicing_key_session(id);
        let dao = SaManager::sa_token_dao();

        if let Some(session) = dao.get_session(&session_key)? {
            Ok(session)
        } else {
            let mut session = SaSession::new(&session_key);
            session.set_login_type(&self.login_type);
            session.set_login_id(id);
            session.set_session_type("login");
            Ok(session)
        }
    }

    // ==================== 登出 ====================

    /// 注销当前会话
    pub fn logout(&self) -> SaResult<()> {
        if let Some(login_id) = self.get_login_id_default_null()? {
            self.logout_by_login_id(&login_id)?;
        }
        Ok(())
    }

    /// 按 loginId 注销
    pub fn logout_by_login_id(&self, login_id: &str) -> SaResult<()> {
        // 获取该账号的所有 Token
        let token_list = self.get_token_value_list_by_login_id(login_id)?;

        // 逐个注销
        for token in token_list {
            self.logout_by_token_value(&token)?;
        }

        // 删除 Session
        let session_key = self.splicing_key_session(login_id);
        SaManager::sa_token_dao().delete_session(&session_key)?;

        Ok(())
    }

    /// 按 Token 注销
    pub fn logout_by_token_value(&self, token_value: &str) -> SaResult<()> {
        // 获取 loginId
        let login_id = self.get_login_id_by_token(token_value)?;
        if login_id.is_none() {
            return Ok(());
        }
        let login_id = login_id.unwrap();

        // 删除 Token-LoginId 映射
        let token_key = self.splicing_key_token_value(token_value);
        SaManager::sa_token_dao().delete(&token_key)?;

        // 删除最后活跃时间
        let active_key = self.splicing_key_last_active_time(token_value);
        SaManager::sa_token_dao().delete(&active_key)?;

        // 从 Session 中移除终端
        let session_key = self.splicing_key_session(&login_id);
        if let Some(mut session) = SaManager::sa_token_dao().get_session(&session_key)? {
            session.remove_terminal(token_value);
            SaManager::sa_token_dao().update_session(&session)?;
        }

        // 触发登出事件
        SaManager::listeners()
            .read()
            .unwrap()
            .iter()
            .for_each(|l| l.do_logout(&self.login_type, &login_id, token_value));

        Ok(())
    }

    /// 踢人下线
    pub fn kickout_by_login_id(&self, login_id: &str) -> SaResult<()> {
        let token_list = self.get_token_value_list_by_login_id(login_id)?;
        for token in token_list {
            self.kickout_by_token_value(&token)?;
        }
        Ok(())
    }

    /// 按 Token 踢人下线
    pub fn kickout_by_token_value(&self, token_value: &str) -> SaResult<()> {
        let login_id = self.get_login_id_by_token(token_value)?;
        if login_id.is_none() {
            return Ok(());
        }
        let login_id = login_id.unwrap();

        // 删除 Token
        let token_key = self.splicing_key_token_value(token_value);
        SaManager::sa_token_dao().delete(&token_key)?;

        // 删除最后活跃时间
        let active_key = self.splicing_key_last_active_time(token_value);
        SaManager::sa_token_dao().delete(&active_key)?;

        // 从 Session 中移除终端
        let session_key = self.splicing_key_session(&login_id);
        if let Some(mut session) = SaManager::sa_token_dao().get_session(&session_key)? {
            session.remove_terminal(token_value);
            SaManager::sa_token_dao().update_session(&session)?;
        }

        // 触发踢人事件
        SaManager::listeners()
            .read()
            .unwrap()
            .iter()
            .for_each(|l| l.do_kickout(&self.login_type, &login_id, token_value));

        Ok(())
    }

    /// 顶人下线
    pub fn replaced_by_login_id(&self, login_id: &str) -> SaResult<()> {
        let token_list = self.get_token_value_list_by_login_id(login_id)?;
        for token in token_list {
            self.replaced_by_token_value(&token)?;
        }
        Ok(())
    }

    /// 按 Token 顶人下线
    pub fn replaced_by_token_value(&self, token_value: &str) -> SaResult<()> {
        let login_id = self.get_login_id_by_token(token_value)?;
        if login_id.is_none() {
            return Ok(());
        }
        let login_id = login_id.unwrap();

        // 触发顶替事件
        SaManager::listeners()
            .read()
            .unwrap()
            .iter()
            .for_each(|l| l.do_replaced(&self.login_type, &login_id, token_value));

        // 删除 Token
        let token_key = self.splicing_key_token_value(token_value);
        SaManager::sa_token_dao().delete(&token_key)?;

        // 删除最后活跃时间
        let active_key = self.splicing_key_last_active_time(token_value);
        SaManager::sa_token_dao().delete(&active_key)?;

        let session_key = self.splicing_key_session(&login_id);
        if let Some(mut session) = SaManager::sa_token_dao().get_session(&session_key)? {
            session.remove_terminal(token_value);
            SaManager::sa_token_dao().update_session(&session)?;
        }

        Ok(())
    }

    // ==================== 登录状态 ====================

    /// 是否已登录
    pub fn is_login(&self) -> SaResult<bool> {
        Ok(self.get_login_id_default_null()?.is_some())
    }

    /// 检查是否已登录（未登录抛异常）
    pub fn check_login(&self) -> SaResult<()> {
        if !self.is_login()? {
            return Err(SaTokenException::not_login("未登录", &self.login_type));
        }
        Ok(())
    }

    /// 获取当前登录 ID
    pub fn get_login_id(&self) -> SaResult<String> {
        self.get_login_id_default_null()?
            .ok_or_else(|| SaTokenException::not_login("未登录", &self.login_type))
    }

    /// 获取当前登录 ID（未登录返回 None）
    pub fn get_login_id_default_null(&self) -> SaResult<Option<String>> {
        let Some(token_value) = self.get_token_value() else {
            return Ok(None);
        };
        self.get_login_id_by_token(&token_value)
    }

    /// 获取当前登录 ID（转为 String）
    pub fn get_login_id_as_string(&self) -> SaResult<String> {
        self.get_login_id()
    }

    /// 获取当前登录 ID（转为 i32）
    pub fn get_login_id_as_i32(&self) -> SaResult<i32> {
        let id = self.get_login_id()?;
        id.parse::<i32>()
            .map_err(|_| SaTokenException::other(format!("无法将 loginId 转换为 i32: {id}")))
    }

    /// 获取当前登录 ID（转为 i64）
    pub fn get_login_id_as_i64(&self) -> SaResult<i64> {
        let id = self.get_login_id()?;
        id.parse::<i64>()
            .map_err(|_| SaTokenException::other(format!("无法将 loginId 转换为 i64: {id}")))
    }

    /// 根据 Token 获取 loginId
    pub fn get_login_id_by_token(&self, token_value: &str) -> SaResult<Option<String>> {
        let token_key = self.splicing_key_token_value(token_value);
        SaManager::sa_token_dao().get(&token_key)
    }

    // ==================== Token ====================

    /// 获取当前 Token 值
    pub fn get_token_value(&self) -> Option<String> {
        let config = self.config();
        let context = SaManager::sa_token_context();

        // 从请求中读取 Token
        if context.is_valid() {
            let req = context.request();

            // 先从 Header 读取
            if config.is_read_header {
                if let Some(token) = req.get_header(&config.token_name) {
                    if !token.is_empty() {
                        return Some(self.cut_token_prefix(&token));
                    }
                }
            }

            // 再从 Cookie 读取
            if config.is_read_cookie {
                if let Some(token) = req.get_cookie_value(&config.token_name) {
                    if !token.is_empty() {
                        return Some(self.cut_token_prefix(&token));
                    }
                }
            }

            // 最后从参数读取
            if config.is_read_body {
                if let Some(token) = req.get_param(&config.token_name) {
                    if !token.is_empty() {
                        return Some(self.cut_token_prefix(&token));
                    }
                }
            }
        }

        // 从 Storage 读取
        if context.is_valid() {
            if let Some(token) = context.storage().get(&config.token_name) {
                if !token.is_empty() {
                    return Some(token);
                }
            }
        }

        None
    }

    /// 获取 Token 名称
    pub fn token_name(&self) -> String {
        self.config().token_name().to_string()
    }

    /// 获取 Token 详情
    pub fn get_token_info(&self) -> SaResult<SaTokenInfo> {
        let token_value = self
            .get_token_value()
            .ok_or_else(|| SaTokenException::not_login("未提供 Token", &self.login_type))?;

        let login_id = self
            .get_login_id_by_token(&token_value)?
            .ok_or_else(|| SaTokenException::not_login("Token 无效", &self.login_type))?;

        let mut info = SaTokenInfo::new(self.token_name(), &token_value);
        info.login_id = login_id;
        info.token_timeout = self.get_token_timeout_by_token(&token_value)?;
        info.login_device_type = self
            .get_login_device_type_by_token(&token_value)?
            .unwrap_or_default();

        Ok(info)
    }

    /// 设置 Token 值
    pub fn set_token_value(&self, token_value: &str) -> SaResult<()> {
        let config = self.config();
        let context = SaManager::sa_token_context();

        if !context.is_valid() {
            return Ok(());
        }

        // 写入 Storage
        context.storage().set(&config.token_name, token_value);

        // 写入 Cookie
        if config.is_read_cookie {
            let cookie =
                crate::context::model::sa_cookie::SaCookie::new(&config.token_name, token_value);
            context.response().add_cookie(cookie);
        }

        // 写入响应头
        if config.is_write_header {
            context
                .response()
                .set_header(&config.token_name, token_value);
        }

        Ok(())
    }

    // ==================== 会话 ====================

    /// 获取当前会话
    pub fn get_session(&self) -> SaResult<SaSession> {
        let login_id = self.get_login_id()?;
        self.get_session_by_login_id(&login_id)
    }

    /// 按 loginId 获取会话
    pub fn get_session_by_login_id(&self, login_id: &str) -> SaResult<SaSession> {
        let session_key = self.splicing_key_session(login_id);
        SaManager::sa_token_dao()
            .get_session(&session_key)?
            .ok_or_else(|| SaTokenException::other("Session 不存在"))
    }

    /// 获取 Token-Session
    pub fn get_token_session(&self) -> SaResult<SaSession> {
        let token_value = self
            .get_token_value()
            .ok_or_else(|| SaTokenException::not_login("未提供 Token", &self.login_type))?;
        self.get_token_session_by_token_create(&token_value, true)?
            .ok_or_else(|| SaTokenException::other("Token-Session 不存在"))
    }

    /// 按 Token 获取 Token-Session
    pub fn get_token_session_by_token(&self, token_value: &str) -> SaResult<SaSession> {
        self.get_token_session_by_token_create(token_value, true)?
            .ok_or_else(|| SaTokenException::other("Token-Session 不存在"))
    }

    /// 按 Token 获取 Token-Session，可控制不存在时是否创建
    ///
    /// 对应 Java `getTokenSessionByToken(tokenValue, isCreate)`。
    pub fn get_token_session_by_token_create(
        &self,
        token_value: &str,
        is_create: bool,
    ) -> SaResult<Option<SaSession>> {
        if token_value.is_empty() {
            return Err(SaTokenException::other("Token-Session 获取失败：token 为空"));
        }

        let session_key = self.splicing_key_token_session(token_value);
        if let Some(session) = SaManager::sa_token_dao().get_session(&session_key)? {
            return Ok(Some(session));
        }
        if !is_create {
            return Ok(None);
        }
        if self.get_login_id_by_token(token_value)?.is_none() {
            return Err(SaTokenException::other(format!(
                "Token-Session 获取失败，token 无效: {token_value}"
            )));
        }

        let mut session = SaSession::new(&session_key);
        session.set_session_type(crate::util::sa_token_consts::SESSION_TYPE_TOKEN);
        session.set_login_type(&self.login_type);
        session.set_token(token_value);
        let timeout = self.get_token_timeout_by_token(token_value)?;
        SaManager::sa_token_dao().set_session(&session, timeout)?;
        Ok(Some(session))
    }

    /// 获取匿名 Token-Session（对应 Java `getAnonTokenSession()`）
    pub fn get_anon_token_session(&self) -> SaResult<SaSession> {
        self.get_anon_token_session_create(true)?
            .ok_or_else(|| SaTokenException::other("Anon Token-Session 不存在"))
    }

    /// 获取匿名 Token-Session，可控制不存在时是否创建
    ///
    /// 对应 Java `getAnonTokenSession(boolean isCreate)`。
    pub fn get_anon_token_session_create(&self, is_create: bool) -> SaResult<Option<SaSession>> {
        if let Some(token_value) = self.get_token_value() {
            if !token_value.is_empty() {
                if let Some(session) = self.get_token_session_by_token_create(&token_value, false)? {
                    return Ok(Some(session));
                }
                if self.get_login_id_by_token(&token_value)?.is_some() {
                    return self.get_token_session_by_token_create(&token_value, is_create);
                }
            }
        }

        if !is_create {
            return Ok(None);
        }

        let token_value = super::shared::create_token_value(&self.config());
        self.set_last_active_to_now(&token_value)?;
        self.set_token_value(&token_value)?;

        let session_key = self.splicing_key_token_session(&token_value);
        let mut session = SaSession::new(&session_key);
        session.set_session_type(crate::util::sa_token_consts::SESSION_TYPE_ANON);
        session.set_login_type(&self.login_type);
        session.set_token(&token_value);
        let timeout = self.config().timeout;
        SaManager::sa_token_dao().set_session(&session, timeout)?;
        Ok(Some(session))
    }

    // ==================== Token 超时 ====================

    /// 获取当前 Token 超时时间
    pub fn get_token_timeout(&self) -> SaResult<i64> {
        if let Some(token_value) = self.get_token_value() {
            self.get_token_timeout_by_token(&token_value)
        } else {
            Ok(-2)
        }
    }

    /// 按 Token 获取超时时间
    pub fn get_token_timeout_by_token(&self, token_value: &str) -> SaResult<i64> {
        let token_key = self.splicing_key_token_value(token_value);
        SaManager::sa_token_dao().get_timeout(&token_key)
    }

    /// 续签当前 Token
    pub fn renew_timeout(&self, timeout: i64) -> SaResult<()> {
        let token_value = self
            .get_token_value()
            .ok_or_else(|| SaTokenException::not_login("未提供 Token", &self.login_type))?;
        self.renew_timeout_by_token(&token_value, timeout)
    }

    /// 续签指定 Token
    pub fn renew_timeout_by_token(&self, token_value: &str, timeout: i64) -> SaResult<()> {
        let token_key = self.splicing_key_token_value(token_value);
        SaManager::sa_token_dao().update_timeout(&token_key, timeout)?;

        // 更新最后活跃时间
        self.set_last_active_to_now(token_value)?;

        Ok(())
    }

    // ==================== 活跃超时 ====================

    /// 更新最后活跃时间为当前
    pub fn set_last_active_to_now(&self, token_value: &str) -> SaResult<()> {
        let active_key = self.splicing_key_last_active_time(token_value);
        let now = crate::util::sa_fox_util::now_timestamp().to_string();
        SaManager::sa_token_dao().set(&active_key, &now, -1)
    }

    /// 获取 Token 最后活跃时间
    pub fn get_token_last_active_time(&self, token_value: &str) -> SaResult<i64> {
        let active_key = self.splicing_key_last_active_time(token_value);
        Ok(SaManager::sa_token_dao()
            .get(&active_key)?
            .and_then(|s| s.parse().ok())
            .unwrap_or(0))
    }

    /// 检查活跃超时
    pub fn check_active_timeout(&self) -> SaResult<()> {
        let config = self.config();
        if config.active_timeout <= 0 {
            return Ok(());
        }

        let token_value = self
            .get_token_value()
            .ok_or_else(|| SaTokenException::not_login("未提供 Token", &self.login_type))?;

        let last_active = self.get_token_last_active_time(&token_value)?;
        let now = crate::util::sa_fox_util::now_timestamp();

        if now - last_active > config.active_timeout {
            // 活跃超时，注销
            self.logout_by_token_value(&token_value)?;
            return Err(SaTokenException::not_login("活跃超时", &self.login_type));
        }

        Ok(())
    }

    // ==================== 设备 ====================

    /// 获取当前登录设备类型
    pub fn get_login_device_type(&self) -> SaResult<String> {
        let token_value = self
            .get_token_value()
            .ok_or_else(|| SaTokenException::not_login("未提供 Token", &self.login_type))?;
        self.get_login_device_type_by_token(&token_value)?
            .ok_or_else(|| SaTokenException::other("无法获取设备类型"))
    }

    /// 按 Token 获取设备类型
    pub fn get_login_device_type_by_token(&self, token_value: &str) -> SaResult<Option<String>> {
        let Some(login_id) = self.get_login_id_by_token(token_value)? else {
            return Ok(None);
        };
        let session_key = self.splicing_key_session(&login_id);
        let Some(session) = SaManager::sa_token_dao().get_session(&session_key)? else {
            return Ok(None);
        };
        Ok(session
            .get_terminal(token_value)
            .map(|t| t.device_type().to_string()))
    }

    /// 获取当前登录设备 ID
    pub fn get_login_device_id(&self) -> SaResult<String> {
        let token_value = self
            .get_token_value()
            .ok_or_else(|| SaTokenException::not_login("未提供 Token", &self.login_type))?;
        self.get_login_device_id_by_token(&token_value)?
            .ok_or_else(|| SaTokenException::other("无法获取设备 ID"))
    }

    /// 按 Token 获取设备 ID
    pub fn get_login_device_id_by_token(&self, token_value: &str) -> SaResult<Option<String>> {
        let Some(login_id) = self.get_login_id_by_token(token_value)? else {
            return Ok(None);
        };
        let session_key = self.splicing_key_session(&login_id);
        let Some(session) = SaManager::sa_token_dao().get_session(&session_key)? else {
            return Ok(None);
        };
        Ok(session
            .get_terminal(token_value)
            .map(|t| t.device_id().to_string()))
    }

    // ==================== 终端查询 ====================

    /// 获取指定账号的终端列表
    pub fn get_terminal_list_by_login_id(&self, login_id: &str) -> SaResult<Vec<SaTerminalInfo>> {
        let session_key = self.splicing_key_session(login_id);
        let session = SaManager::sa_token_dao()
            .get_session(&session_key)?
            .ok_or_else(|| SaTokenException::other("Session 不存在"))?;
        Ok(session.terminal_list().to_vec())
    }

    /// 获取当前终端信息
    pub fn get_terminal_info(&self) -> SaResult<SaTerminalInfo> {
        let token_value = self
            .get_token_value()
            .ok_or_else(|| SaTokenException::not_login("未提供 Token", &self.login_type))?;
        self.get_terminal_info_by_token(&token_value)
    }

    /// 按 Token 获取终端信息
    pub fn get_terminal_info_by_token(&self, token_value: &str) -> SaResult<SaTerminalInfo> {
        let login_id = self
            .get_login_id_by_token(token_value)?
            .ok_or_else(|| SaTokenException::not_login("Token 无效", &self.login_type))?;
        let session_key = self.splicing_key_session(&login_id);
        let session = SaManager::sa_token_dao()
            .get_session(&session_key)?
            .ok_or_else(|| SaTokenException::other("Session 不存在"))?;
        session
            .get_terminal(token_value)
            .cloned()
            .ok_or_else(|| SaTokenException::other("终端不存在"))
    }

    /// 获取指定账号的 Token 列表
    pub fn get_token_value_list_by_login_id(&self, login_id: &str) -> SaResult<Vec<String>> {
        let session_key = self.splicing_key_session(login_id);
        if let Some(session) = SaManager::sa_token_dao().get_session(&session_key)? {
            Ok(session
                .terminal_list()
                .iter()
                .map(|t| t.token_value().to_string())
                .collect())
        } else {
            Ok(Vec::new())
        }
    }

    /// 获取指定账号的 Token 值
    pub fn get_token_value_by_login_id(&self, login_id: &str) -> SaResult<Option<String>> {
        Ok(self
            .get_token_value_list_by_login_id(login_id)?
            .into_iter()
            .next())
    }

    // ==================== Key 拼接 ====================

    /// 拼接 Token 名称 Key
    pub fn splicing_key_token_name(&self) -> String {
        format!("satoken:login:token-name:{}", self.login_type)
    }

    /// 拼接 Token 值 Key
    pub fn splicing_key_token_value(&self, token_value: &str) -> String {
        super::shared::token_key(&self.config().token_name, &self.login_type, token_value)
    }

    /// 拼接 Session Key
    pub fn splicing_key_session(&self, login_id: &str) -> String {
        super::shared::session_key(&self.config().token_name, &self.login_type, login_id)
    }

    /// 拼接 Token-Session Key
    pub fn splicing_key_token_session(&self, token_value: &str) -> String {
        super::shared::token_session_key(&self.config().token_name, &self.login_type, token_value)
    }

    /// 拼接最后活跃时间 Key
    pub fn splicing_key_last_active_time(&self, token_value: &str) -> String {
        super::shared::last_active_key(&self.config().token_name, &self.login_type, token_value)
    }

    /// 拼接封禁 Key
    pub fn splicing_key_disable(&self, login_id: &str, service: &str) -> String {
        super::shared::disable_key(
            &self.config().token_name,
            &self.login_type,
            login_id,
            service,
        )
    }

    /// 拼接二级认证 Key
    pub fn splicing_key_safe(&self, token_value: &str, service: &str) -> String {
        super::shared::safe_key(
            &self.config().token_name,
            &self.login_type,
            token_value,
            service,
        )
    }

    /// 拼接切换账号 Key
    pub fn splicing_key_switch(&self) -> String {
        super::shared::switch_key(&self.login_type)
    }

    // ==================== 权限 / 角色 ====================

    /// 获取当前账号角色列表
    pub fn get_role_list(&self) -> SaResult<Vec<String>> {
        let login_id = self.get_login_id()?;
        self.get_role_list_for(&login_id)
    }

    /// 获取指定账号角色列表
    pub fn get_role_list_for(&self, login_id: &str) -> SaResult<Vec<String>> {
        Ok(SaManager::stp_interface().get_role_list(login_id, &self.login_type))
    }

    /// 当前账号是否具有指定角色
    pub fn has_role(&self, role: &str) -> SaResult<bool> {
        let login_id = self.get_login_id()?;
        self.has_role_for(&login_id, role)
    }

    /// 指定账号是否具有指定角色
    pub fn has_role_for(&self, login_id: &str, role: &str) -> SaResult<bool> {
        let role_list = self.get_role_list_for(login_id)?;
        Ok(role_list.contains(&role.to_string()))
    }

    /// 当前账号是否具有指定角色（AND 模式）
    pub fn has_role_and(&self, roles: &[&str]) -> SaResult<bool> {
        let login_id = self.get_login_id()?;
        let role_list = self.get_role_list_for(&login_id)?;
        Ok(roles.iter().all(|r| role_list.contains(&r.to_string())))
    }

    /// 当前账号是否具有指定角色（OR 模式）
    pub fn has_role_or(&self, roles: &[&str]) -> SaResult<bool> {
        let login_id = self.get_login_id()?;
        let role_list = self.get_role_list_for(&login_id)?;
        Ok(roles.iter().any(|r| role_list.contains(&r.to_string())))
    }

    /// 检查当前账号是否具有指定角色（不满足则抛异常）
    pub fn check_role(&self, role: &str) -> SaResult<()> {
        if !self.has_role(role)? {
            return Err(SaTokenException::not_role(role, &self.login_type));
        }
        Ok(())
    }

    /// 检查当前账号是否具有指定角色（AND 模式）
    pub fn check_role_and(&self, roles: &[&str]) -> SaResult<()> {
        if !self.has_role_and(roles)? {
            return Err(SaTokenException::not_role(
                roles.join(","),
                &self.login_type,
            ));
        }
        Ok(())
    }

    /// 检查当前账号是否具有指定角色（OR 模式）
    pub fn check_role_or(&self, roles: &[&str]) -> SaResult<()> {
        if !self.has_role_or(roles)? {
            return Err(SaTokenException::not_role(
                roles.join(","),
                &self.login_type,
            ));
        }
        Ok(())
    }

    /// 获取当前账号权限列表
    pub fn get_permission_list(&self) -> SaResult<Vec<String>> {
        let login_id = self.get_login_id()?;
        self.get_permission_list_for(&login_id)
    }

    /// 获取指定账号权限列表
    pub fn get_permission_list_for(&self, login_id: &str) -> SaResult<Vec<String>> {
        Ok(SaManager::stp_interface().get_permission_list(login_id, &self.login_type))
    }

    /// 当前账号是否具有指定权限
    pub fn has_permission(&self, permission: &str) -> SaResult<bool> {
        let login_id = self.get_login_id()?;
        self.has_permission_for(&login_id, permission)
    }

    /// 指定账号是否具有指定权限
    pub fn has_permission_for(&self, login_id: &str, permission: &str) -> SaResult<bool> {
        let permission_list = self.get_permission_list_for(login_id)?;
        Ok(permission_list.contains(&permission.to_string()))
    }

    /// 当前账号是否具有指定权限（AND 模式）
    pub fn has_permission_and(&self, permissions: &[&str]) -> SaResult<bool> {
        let login_id = self.get_login_id()?;
        let permission_list = self.get_permission_list_for(&login_id)?;
        Ok(permissions
            .iter()
            .all(|p| permission_list.contains(&p.to_string())))
    }

    /// 当前账号是否具有指定权限（OR 模式）
    pub fn has_permission_or(&self, permissions: &[&str]) -> SaResult<bool> {
        let login_id = self.get_login_id()?;
        let permission_list = self.get_permission_list_for(&login_id)?;
        Ok(permissions
            .iter()
            .any(|p| permission_list.contains(&p.to_string())))
    }

    /// 检查当前账号是否具有指定权限（不满足则抛异常）
    pub fn check_permission(&self, permission: &str) -> SaResult<()> {
        if !self.has_permission(permission)? {
            return Err(SaTokenException::not_permission(
                permission,
                &self.login_type,
            ));
        }
        Ok(())
    }

    /// 检查当前账号是否具有指定权限（AND 模式）
    pub fn check_permission_and(&self, permissions: &[&str]) -> SaResult<()> {
        if !self.has_permission_and(permissions)? {
            return Err(SaTokenException::not_permission(
                permissions.join(","),
                &self.login_type,
            ));
        }
        Ok(())
    }

    /// 检查当前账号是否具有指定权限（OR 模式）
    pub fn check_permission_or(&self, permissions: &[&str]) -> SaResult<()> {
        if !self.has_permission_or(permissions)? {
            return Err(SaTokenException::not_permission(
                permissions.join(","),
                &self.login_type,
            ));
        }
        Ok(())
    }

    // ==================== 禁用 ====================

    /// 封禁账号（默认服务 `login`，默认等级 1）
    pub fn disable(&self, login_id: &str, time: i64) -> SaResult<()> {
        self.disable_level(login_id, crate::util::sa_token_consts::DEFAULT_DISABLE_LEVEL, time)
    }

    /// 封禁账号指定服务（默认等级 1）
    pub fn disable_with_service(&self, login_id: &str, service: &str, time: i64) -> SaResult<()> {
        self.disable_level_with_service(
            login_id,
            service,
            crate::util::sa_token_consts::DEFAULT_DISABLE_LEVEL,
            time,
        )
    }

    /// 阶梯封禁：指定账号与等级
    pub fn disable_level(&self, login_id: &str, level: i32, time: i64) -> SaResult<()> {
        self.disable_level_with_service(
            login_id,
            crate::util::sa_token_consts::DEFAULT_DISABLE_SERVICE,
            level,
            time,
        )
    }

    /// 阶梯封禁：指定账号、服务与等级
    pub fn disable_level_with_service(
        &self,
        login_id: &str,
        service: &str,
        level: i32,
        time: i64,
    ) -> SaResult<()> {
        self.validate_disable_args(login_id, service, level)?;
        let key = self.splicing_key_disable(login_id, service);
        SaManager::sa_token_dao().set(&key, &level.to_string(), time)?;
        SaManager::listeners()
            .read()
            .map_err(|_| SaTokenException::other("监听器读锁已损坏"))?
            .iter()
            .for_each(|listener| listener.do_disable(&self.login_type, login_id, service, level, time));
        Ok(())
    }

    /// 解封账号
    pub fn untie_disable(&self, login_id: &str) -> SaResult<()> {
        self.untie_disable_with_service(login_id, crate::util::sa_token_consts::DEFAULT_DISABLE_SERVICE)
    }

    /// 解封账号指定服务
    pub fn untie_disable_with_service(&self, login_id: &str, service: &str) -> SaResult<()> {
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
        let key = self.splicing_key_disable(login_id, service);
        SaManager::sa_token_dao().delete(&key)?;
        SaManager::listeners()
            .read()
            .map_err(|_| SaTokenException::other("监听器读锁已损坏"))?
            .iter()
            .for_each(|listener| listener.do_untie_disable(&self.login_type, login_id, service));
        Ok(())
    }

    /// 获取封禁剩余时间（默认服务）
    pub fn get_disable_time(&self, login_id: &str) -> SaResult<i64> {
        self.get_disable_time_with_service(login_id, crate::util::sa_token_consts::DEFAULT_DISABLE_SERVICE)
    }

    /// 获取指定服务封禁剩余时间
    pub fn get_disable_time_with_service(&self, login_id: &str, service: &str) -> SaResult<i64> {
        let key = self.splicing_key_disable(login_id, service);
        SaManager::sa_token_dao().get_timeout(&key)
    }

    /// 获取封禁等级（默认服务）
    pub fn get_disable_level(&self, login_id: &str) -> SaResult<i32> {
        self.get_disable_level_with_service(
            login_id,
            crate::util::sa_token_consts::DEFAULT_DISABLE_SERVICE,
        )
    }

    /// 获取指定服务封禁等级，未封禁返回 `NOT_DISABLE_LEVEL`
    pub fn get_disable_level_with_service(&self, login_id: &str, service: &str) -> SaResult<i32> {
        let key = self.splicing_key_disable(login_id, service);
        if let Some(value) = SaManager::sa_token_dao().get(&key)? {
            return Ok(value.parse().unwrap_or(crate::util::sa_token_consts::DEFAULT_DISABLE_LEVEL));
        }

        let wrapper = SaManager::stp_interface().is_disabled(login_id, service);
        if wrapper.disable_time == crate::util::sa_token_consts::NEVER_EXPIRE
            || wrapper.disable_time > 0
        {
            self.disable_level_with_service(
                login_id,
                service,
                wrapper.disable_level,
                wrapper.disable_time,
            )?;
        }
        Ok(wrapper.disable_level)
    }

    /// 指定账号是否被封禁
    pub fn is_disable(&self, login_id: &str) -> SaResult<bool> {
        self.is_disable_level_with_service(
            login_id,
            crate::util::sa_token_consts::DEFAULT_DISABLE_SERVICE,
            crate::util::sa_token_consts::MIN_DISABLE_LEVEL,
        )
    }

    /// 指定账号指定服务是否被封禁
    pub fn is_disable_with_service(&self, login_id: &str, service: &str) -> SaResult<bool> {
        self.is_disable_level_with_service(
            login_id,
            service,
            crate::util::sa_token_consts::MIN_DISABLE_LEVEL,
        )
    }

    /// 是否被封禁到指定等级
    pub fn is_disable_level(&self, login_id: &str, level: i32) -> SaResult<bool> {
        self.is_disable_level_with_service(
            login_id,
            crate::util::sa_token_consts::DEFAULT_DISABLE_SERVICE,
            level,
        )
    }

    /// 指定服务是否被封禁到指定等级
    pub fn is_disable_level_with_service(
        &self,
        login_id: &str,
        service: &str,
        level: i32,
    ) -> SaResult<bool> {
        let disable_level = self.get_disable_level_with_service(login_id, service)?;
        if disable_level == crate::util::sa_token_consts::NOT_DISABLE_LEVEL {
            return Ok(false);
        }
        Ok(disable_level >= level)
    }

    /// 检查账号是否被封禁（被封禁则抛异常）
    pub fn check_disable(&self, login_id: &str) -> SaResult<()> {
        self.check_disable_level_with_service(
            login_id,
            crate::util::sa_token_consts::DEFAULT_DISABLE_SERVICE,
            crate::util::sa_token_consts::MIN_DISABLE_LEVEL,
        )
    }

    /// 检查指定服务是否被封禁
    pub fn check_disable_with_service(&self, login_id: &str, service: &str) -> SaResult<()> {
        self.check_disable_level_with_service(
            login_id,
            service,
            crate::util::sa_token_consts::MIN_DISABLE_LEVEL,
        )
    }

    /// 检查是否被封禁到指定等级
    pub fn check_disable_level(&self, login_id: &str, level: i32) -> SaResult<()> {
        self.check_disable_level_with_service(
            login_id,
            crate::util::sa_token_consts::DEFAULT_DISABLE_SERVICE,
            level,
        )
    }

    /// 检查指定服务是否被封禁到指定等级
    pub fn check_disable_level_with_service(
        &self,
        login_id: &str,
        service: &str,
        level: i32,
    ) -> SaResult<()> {
        let disable_level = self.get_disable_level_with_service(login_id, service)?;
        if disable_level == crate::util::sa_token_consts::NOT_DISABLE_LEVEL {
            return Ok(());
        }
        if disable_level >= level {
            let disable_time = self.get_disable_time_with_service(login_id, service)?;
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

    /// 校验封禁参数（对应 Java `disableLevel` 前置检查）
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

    // ==================== 安全认证 ====================

    /// 开启二级认证
    pub fn open_safe(&self, safe_time: i64) -> SaResult<()> {
        self.open_safe_with_service("", safe_time)
    }

    /// 开启二级认证（指定业务）
    pub fn open_safe_with_service(&self, service: &str, safe_time: i64) -> SaResult<()> {
        let token_value = self
            .get_token_value()
            .ok_or_else(|| SaTokenException::not_login("未提供 Token", &self.login_type))?;
        let key = self.splicing_key_safe(&token_value, service);
        SaManager::sa_token_dao().set(&key, "SAFE_AUTH_SAVE_VALUE", safe_time)?;
        SaManager::listeners()
            .read()
            .map_err(|_| SaTokenException::other("监听器读锁已损坏"))?
            .iter()
            .for_each(|listener| {
                listener.do_open_safe(&self.login_type, &token_value, service, safe_time)
            });
        Ok(())
    }

    /// 当前是否处于二级认证
    pub fn is_safe(&self) -> SaResult<bool> {
        self.is_safe_with_service("")
    }

    /// 指定业务是否处于二级认证
    pub fn is_safe_with_service(&self, service: &str) -> SaResult<bool> {
        if let Some(token_value) = self.get_token_value() {
            let key = self.splicing_key_safe(&token_value, service);
            Ok(SaManager::sa_token_dao().get(&key)?.is_some())
        } else {
            Ok(false)
        }
    }

    /// 校验二级认证（未通过则抛异常）
    pub fn check_safe(&self) -> SaResult<()> {
        self.check_safe_with_service("")
    }

    /// 校验指定业务的二级认证
    pub fn check_safe_with_service(&self, service: &str) -> SaResult<()> {
        if !self.is_safe_with_service(service)? {
            return Err(SaTokenException::not_safe(service, &self.login_type));
        }
        Ok(())
    }

    /// 关闭二级认证
    pub fn close_safe(&self) -> SaResult<()> {
        self.close_safe_with_service("")
    }

    /// 关闭指定业务的二级认证
    pub fn close_safe_with_service(&self, service: &str) -> SaResult<()> {
        if let Some(token_value) = self.get_token_value() {
            let key = self.splicing_key_safe(&token_value, service);
            SaManager::sa_token_dao().delete(&key)?;
            SaManager::listeners()
                .read()
                .map_err(|_| SaTokenException::other("监听器读锁已损坏"))?
                .iter()
                .for_each(|listener| {
                    listener.do_close_safe(&self.login_type, &token_value, service)
                });
        }
        Ok(())
    }

    // ==================== 切换账号 ====================

    /// 临时切换到指定账号身份
    pub fn switch_to(&self, login_id: &str) -> SaResult<()> {
        let _token_value = self
            .get_token_value()
            .ok_or_else(|| SaTokenException::not_login("未提供 Token", &self.login_type))?;
        let key = self.splicing_key_switch();
        SaManager::sa_token_context().storage().set(&key, login_id);
        Ok(())
    }

    /// 结束切换
    pub fn end_switch(&self) -> SaResult<()> {
        let key = self.splicing_key_switch();
        SaManager::sa_token_context().storage().delete(&key);
        Ok(())
    }

    /// 当前是否处于切换状态
    pub fn is_switch(&self) -> SaResult<bool> {
        let key = self.splicing_key_switch();
        Ok(SaManager::sa_token_context().storage().get(&key).is_some())
    }

    /// 获取临时切换的 loginId
    pub fn get_switch_login_id(&self) -> SaResult<Option<String>> {
        let key = self.splicing_key_switch();
        Ok(SaManager::sa_token_context().storage().get(&key))
    }

    // ==================== 内部方法 ====================

    /// 生成 Token 值
    fn create_token_value(&self, _id: &str, _param: &SaLoginParameter) -> String {
        let config = self.config();
        super::shared::create_token_value(&config)
    }

    /// 裁剪 Token 前缀
    fn cut_token_prefix(&self, token: &str) -> String {
        let config = self.config();
        let prefix = &config.token_prefix;
        if !prefix.is_empty() && token.starts_with(prefix.as_str()) {
            token[prefix.len()..].to_string()
        } else {
            token.to_string()
        }
    }
}
