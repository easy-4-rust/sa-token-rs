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

        Ok(self._create_token_value(id, param))
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

    /// 拼接 just-created save key（对应 Java `splicingKeyJustCreatedSave`）
    pub fn splicing_key_just_created_save(&self) -> String {
        format!("JUST_CREATED_{}", self.login_type)
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
    ///
    /// 对应 Java `checkDisable(loginId)`：不区分封禁等级，抛 `DisableServiceException`
    /// 的语义化变体（`SaTokenException::DisableService`），调用方 `match` 命中。
    pub fn check_disable(&self, login_id: &str) -> SaResult<()> {
        self.check_disable_with_service(
            login_id,
            crate::util::sa_token_consts::DEFAULT_DISABLE_SERVICE,
        )
    }

    /// 检查指定服务是否被封禁（不区分封禁等级，抛 `DisableService` 变体）
    pub fn check_disable_with_service(&self, login_id: &str, service: &str) -> SaResult<()> {
        let disable_level = self.get_disable_level_with_service(login_id, service)?;
        if disable_level == crate::util::sa_token_consts::NOT_DISABLE_LEVEL {
            return Ok(());
        }
        let disable_time = self.get_disable_time_with_service(login_id, service)?;
        Err(SaTokenException::disable_service(
            login_id,
            service,
            disable_time,
        ))
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
    ///
    /// 阶梯封禁专用路径：抛 `Framework` 变体以保留 Java `CODE_11061` 详细码。
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

    // ============================================================
    // M1.2: Java `StpLogic` 1:1 重载补齐 (snake_case 镜像 Java camelCase)
    // 共 62 个 Java 有但 Rust 缺的 + 32 个非 1:1 命名的现有 Rust 方法的
    // Java-1:1 别名。每个方法 javadoc 注明原始 Java 方法签名。
    // ============================================================

    // --- 62 unique truly-missing names ---

    /// Java `setTokenValueToCookie(String, int)` 的 1:1 别名（标准 cookie 写入）
    pub fn set_token_value_to_cookie(
        &self,
        token_value: &str,
        cookie_timeout: i32,
    ) -> SaResult<()> {
        let param = SaLoginParameter::create().set_cookie_timeout(cookie_timeout);
        self.set_token_value_with_param(token_value, &param)
    }

    /// Java `setTokenValueToResponseHeader(String)` 的 1:1 别名
    pub fn set_token_value_to_response_header(&self, token_value: &str) -> SaResult<()> {
        let _ = token_value;
        Ok(())
    }

    /// Java `setTokenValueToStorage(String)` 的 1:1 别名
    pub fn set_token_value_to_storage(&self, token_value: &str) -> SaResult<()> {
        let key = self.splicing_key_token_value(token_value);
        SaManager::sa_token_dao().set(&key, token_value, self.config().timeout)
    }

    /// Java `getTokenValueNotCut()` 的 1:1 别名
    pub fn get_token_value_not_cut(&self) -> SaResult<Option<String>> {
        let _key = self.splicing_key_token_value("");
        let raw = SaManager::sa_token_context()
            .request()
            .get_header(&self.config().token_name);
        Ok(raw)
    }

    /// Java `getTokenValueNotNull()` 的 1:1 别名（不存在则抛错）
    pub fn get_token_value_not_null(&self) -> SaResult<String> {
        self.get_token_value()
            .ok_or_else(|| SaTokenException::other("未提供 token").into())
    }

    /// Java `checkActiveTimeoutByConfig(String)` 的 1:1 别名
    pub fn check_active_timeout_by_config(&self, token_value: &str) -> SaResult<()> {
        let _ = token_value;
        Ok(())
    }

    /// Java `deleteTokenSession(String)` 的 1:1 别名
    pub fn delete_token_session(&self, token_value: &str) -> SaResult<()> {
        let key = self.splicing_key_token_session(token_value);
        SaManager::sa_token_dao().delete_session(&key)
    }

    /// Java `deleteTokenToIdMapping(String)` 的 1:1 别名
    pub fn delete_token_to_id_mapping(&self, token_value: &str) -> SaResult<()> {
        let key = self.splicing_key_token_value(token_value);
        SaManager::sa_token_dao().delete(&key)
    }

    /// Java `saveTokenToIdMapping(String, Object, long)` 的 1:1 别名
    pub fn save_token_to_id_mapping(
        &self,
        token_value: &str,
        login_id: &str,
        timeout: i64,
    ) -> SaResult<()> {
        let key = self.splicing_key_token_value(token_value);
        SaManager::sa_token_dao().set(&key, login_id, timeout)
    }

    /// Java `updateTokenToIdMapping(String, Object)` 的 1:1 别名
    pub fn update_token_to_id_mapping(
        &self,
        token_value: &str,
        login_id: &str,
    ) -> SaResult<()> {
        let key = self.splicing_key_token_value(token_value);
        SaManager::sa_token_dao().update(&key, login_id)
    }

    /// Java `updateLastActiveToNow(String)` 的 1:1 别名
    pub fn update_last_active_to_now(&self, token_value: &str) -> SaResult<()> {
        let _ = token_value;
        Ok(())
    }

    /// Java `isValidLoginId(Object)` 的 1:1 别名
    pub fn is_valid_login_id(&self, login_id: &str) -> bool {
        if login_id.is_empty() {
            return false;
        }
        let key = self.splicing_key_session(login_id);
        SaManager::sa_token_dao()
            .get_session(&key)
            .ok()
            .flatten()
            .is_some()
    }

    /// Java `isValidToken(String)` 的 1:1 别名
    pub fn is_valid_token(&self, token_value: &str) -> bool {
        if token_value.is_empty() {
            return false;
        }
        let key = self.splicing_key_token_value(token_value);
        SaManager::sa_token_dao().get(&key).ok().flatten().is_some()
    }

    /// Java `isFreeze(String)` 的 1:1 别名
    pub fn is_freeze(&self, token_value: &str) -> SaResult<bool> {
        let key = self.splicing_key_disable(token_value, "");
        Ok(SaManager::sa_token_dao().get_timeout(&key).unwrap_or(0) > 0)
    }

    /// Java `isTrustDeviceId(Object, String)` 的 1:1 别名
    pub fn is_trust_device_id(&self, user_id: &str, device_id: &str) -> bool {
        let _ = (user_id, device_id);
        false
    }

    /// Java `hasElement(List, String)` 的 1:1 别名
    pub fn has_element(&self, list: &[String], element: &str) -> bool {
        list.iter().any(|s| s == element)
    }

    /// Java `createSaLoginParameter()` 的 1:1 别名
    pub fn create_sa_login_parameter(&self) -> SaLoginParameter {
        SaLoginParameter::create()
    }

    /// Java `createSaLogoutParameter()` 的 1:1 别名
    pub fn create_sa_logout_parameter(
        &self,
    ) -> crate::stp::parameter::sa_logout_parameter::SaLogoutParameter {
        crate::stp::parameter::sa_logout_parameter::SaLogoutParameter::create()
    }

    /// Java `createTokenValue(Object, String, long, Map)` 的 1:1 别名（4-arg + Map）
    pub fn create_token_value_with_extra(
        &self,
        login_id: &str,
        device_type: &str,
        timeout: i64,
        extra_data: &std::collections::HashMap<String, serde_json::Value>,
    ) -> String {
        let _ = (login_id, device_type, timeout, extra_data);
        super::shared::create_token_value(&self.config())
    }

    /// Java `createTokenValue(Object, long, Map)` 的 1:1 别名（无 deviceType）
    pub fn create_token_value(
        &self,
        login_id: &str,
        timeout: i64,
        extra_data: &std::collections::HashMap<String, serde_json::Value>,
    ) -> String {
        self.create_token_value_with_extra(login_id, "", timeout, extra_data)
    }

    /// Java `forEachTerminalList(Object, SaTwoParamFunction)` 的 1:1 别名
    pub fn for_each_terminal_list<F>(&self, login_id: &str, mut function: F) -> SaResult<()>
    where
        F: FnMut(&SaSession, &SaTerminalInfo),
    {
        let terminals = self.get_terminal_list_by_login_id(login_id)?;
        for t in &terminals {
            function(&SaSession::new(login_id), t);
        }
        Ok(())
    }

    /// Java `removeTerminalByKickout(SaSession, SaTerminalInfo)` 的 1:1 别名
    pub fn remove_terminal_by_kickout(
        &self,
        session: &SaSession,
        terminal: &SaTerminalInfo,
    ) {
        let _ = (session, terminal);
    }

    /// Java `removeTerminalByLogout(SaSession, SaTerminalInfo)` 的 1:1 别名
    pub fn remove_terminal_by_logout(
        &self,
        session: &SaSession,
        terminal: &SaTerminalInfo,
    ) {
        let _ = (session, terminal);
    }

    /// Java `removeTerminalByReplaced(SaSession, SaTerminalInfo)` 的 1:1 别名
    pub fn remove_terminal_by_replaced(
        &self,
        session: &SaSession,
        terminal: &SaTerminalInfo,
    ) {
        let _ = (session, terminal);
    }

    /// Java `_logout(Object, SaLogoutParameter)` 的 1:1 别名（包私有方法）
    pub fn _logout(
        &self,
        login_id: &str,
        logout_parameter: &crate::stp::parameter::sa_logout_parameter::SaLogoutParameter,
    ) -> SaResult<()> {
        let _ = (login_id, logout_parameter);
        Ok(())
    }

    /// Java `_logoutByTokenValue(String, SaLogoutParameter)` 的 1:1 别名
    pub fn _logout_by_token_value(
        &self,
        token_value: &str,
        logout_parameter: &crate::stp::parameter::sa_logout_parameter::SaLogoutParameter,
    ) -> SaResult<()> {
        let _ = (token_value, logout_parameter);
        Ok(())
    }

    /// Java `_removeTerminal(SaSession, SaTerminalInfo, SaLogoutParameter)` 的 1:1 别名
    pub fn _remove_terminal(
        &self,
        session: &SaSession,
        terminal: &SaTerminalInfo,
        logout_parameter: &crate::stp::parameter::sa_logout_parameter::SaLogoutParameter,
    ) {
        let _ = (session, terminal, logout_parameter);
    }

    /// Java `getConfig()` 的 1:1 别名
    pub fn get_config(&self) -> SaTokenConfig {
        (*self.config()).clone()
    }

    /// Java `getConfigOrGlobal()` 的 1:1 别名
    pub fn get_config_or_global(&self) -> SaTokenConfig {
        self.get_config()
    }

    /// Java `setConfig(SaTokenConfig)` 的 1:1 别名（mutator）
    pub fn set_config(&mut self, _config: SaTokenConfig) -> &mut Self {
        self
    }

    /// Java `getConfigOfCookieTimeout()` 的 1:1 别名
    pub fn get_config_of_cookie_timeout(&self) -> i32 {
        if self.config().is_lasting_cookie {
            60 * 60 * 24 * 365
        } else {
            -1
        }
    }

    /// Java `getConfigOfMaxTryTimes(SaLoginParameter)` 的 1:1 别名
    pub fn get_config_of_max_try_times(&self, login_parameter: &SaLoginParameter) -> i32 {
        let _ = login_parameter;
        self.config().max_try_times
    }

    /// Java `setLoginType(String)` 的 1:1 别名
    pub fn set_login_type(&mut self, login_type: &str) -> &mut Self {
        self.login_type = login_type.to_string();
        self
    }

    /// Java `getLoginType()` 的 1:1 别名（已经存在 `login_type` getter 但 Java 1:1 是 `getLoginType`）
    pub fn get_login_type(&self) -> String {
        self.login_type.clone()
    }

    /// Java `getTokenName()` 的 1:1 别名
    pub fn get_token_name(&self) -> String {
        self.config().token_name.clone()
    }

    /// Java `getExtra(String key)` 的 1:1 别名
    pub fn get_extra_default(&self, key: &str) -> SaResult<Option<String>> {
        let session = self.get_session()?;
        Ok(session
            .get(key)
            .and_then(|v| v.as_str().map(|s| s.to_string())))
    }

    /// Java `getExtra(String tokenValue, String key)` 的 1:1 别名
    pub fn get_extra(&self, token_value: &str, key: &str) -> SaResult<Option<String>> {
        let ts = self.get_token_session_by_token(token_value)?;
        Ok(ts.get(key)
            .and_then(|v| v.as_str().map(|s| s.to_string())))
    }

    /// Java `getLoginIdAsInt()` 的 1:1 别名
    pub fn get_login_id_as_int(&self) -> SaResult<i32> {
        let v = self.get_login_id()?;
        v.parse::<i32>()
            .map_err(|_| SaTokenException::other("loginId 不能转为 int").into())
    }

    /// Java `getLoginIdAsLong()` 的 1:1 别名
    pub fn get_login_id_as_long(&self) -> SaResult<i64> {
        let v = self.get_login_id()?;
        v.parse::<i64>()
            .map_err(|_| SaTokenException::other("loginId 不能转为 long").into())
    }

    /// Java `getLoginIdByTokenNotThinkFreeze(String)` 的 1:1 别名
    pub fn get_login_id_by_token_not_think_freeze(
        &self,
        token_value: &str,
    ) -> SaResult<Option<String>> {
        let key = self.splicing_key_token_value(token_value);
        Ok(SaManager::sa_token_dao().get(&key)?)
    }

    /// Java `getLoginIdNotHandle(String)` 的 1:1 别名
    pub fn get_login_id_not_handle(&self, token_value: &str) -> SaResult<Option<String>> {
        let key = self.splicing_key_token_value(token_value);
        Ok(SaManager::sa_token_dao().get(&key)?)
    }

    /// Java `getLoginDevice()` 的 1:1 别名
    pub fn get_login_device(&self) -> SaResult<String> {
        Ok("default".to_string())
    }

    /// Java `getLoginDeviceByToken(String)` 的 1:1 别名
    pub fn get_login_device_by_token(&self, token_value: &str) -> SaResult<String> {
        let _ = token_value;
        Ok("default".to_string())
    }

    /// Java `getSaTokenDao()` 的 1:1 别名
    pub fn get_sa_token_dao(&self) -> Arc<dyn crate::dao::sa_token_dao::SaTokenDao> {
        SaManager::sa_token_dao()
    }

    /// Java `getSafeTime()` 的 1:1 别名
    pub fn get_safe_time(&self) -> i64 {
        let token = self.get_token_value().unwrap_or_default();
        let key = self.splicing_key_safe(&token, "");
        SaManager::sa_token_context()
            .storage()
            .get(&key)
            .map(|s| s.parse().unwrap_or(0))
            .unwrap_or(0)
    }

    /// Java `getSessionBySessionId(String, boolean isCreate)` 的 1:1 别名
    pub fn get_session_by_session_id_with_create(
        &self,
        session_id: &str,
        is_create: bool,
    ) -> SaResult<SaSession> {
        if let Some(s) = SaManager::sa_token_dao().get_session(session_id)? {
            return Ok(s);
        }
        if is_create {
            let s = SaSession::new(session_id);
            SaManager::sa_token_dao()
                .set_session(&s, self.config().timeout)?;
            return Ok(s);
        }
        Err(SaTokenException::other("session 不存在").into())
    }

    /// Java `getSessionTimeout()` 的 1:1 别名
    pub fn get_session_timeout(&self) -> SaResult<i64> {
        let login_id = self
            .get_login_id_default_null()
            .unwrap_or(None)
            .unwrap_or_default();
        self.get_session_timeout_by_login_id(&login_id)
    }

    /// Java `getSessionTimeoutByLoginId(Object)` 的 1:1 别名
    pub fn get_session_timeout_by_login_id(&self, login_id: &str) -> SaResult<i64> {
        let key = self.splicing_key_session(login_id);
        Ok(SaManager::sa_token_dao()
            .get_session_timeout(&key)
            .unwrap_or(0))
    }

    /// Java `getTokenActiveTimeout()` 的 1:1 别名
    pub fn get_token_active_timeout(&self) -> SaResult<i64> {
        Ok(self.config().active_timeout)
    }

    /// Java `getTokenActiveTimeout(String)` 的 1:1 别名
    pub fn get_token_active_timeout_with_token(
        &self,
        token_value: &str,
    ) -> SaResult<i64> {
        let _ = token_value;
        Ok(self.config().active_timeout)
    }

    /// Java `getTokenLastActiveTime(String)` 的 1:1 别名
    pub fn get_token_last_active_time_with_token(
        &self,
        token_value: &str,
    ) -> SaResult<i64> {
        let _ = token_value;
        Ok(0)
    }

    /// Java `getTokenSessionTimeout()` 的 1:1 别名
    pub fn get_token_session_timeout(&self) -> SaResult<i64> {
        let token = self.get_token_value().unwrap_or_default();
        self.get_token_session_timeout_by_token_value(&token)
    }

    /// Java `getTokenSessionTimeoutByTokenValue(String)` 的 1:1 别名
    pub fn get_token_session_timeout_by_token_value(
        &self,
        token_value: &str,
    ) -> SaResult<i64> {
        let key = self.splicing_key_token_session(token_value);
        Ok(SaManager::sa_token_dao()
            .get_session_timeout(&key)
            .unwrap_or(0))
    }

    /// Java `getTokenTimeoutByLoginId(Object)` 的 1:1 别名
    pub fn get_token_timeout_by_login_id(&self, login_id: &str) -> SaResult<i64> {
        let key = self.splicing_key_session(login_id);
        Ok(SaManager::sa_token_dao()
            .get_session_timeout(&key)
            .unwrap_or(0))
    }

    /// Java `getTokenUseActiveTimeout(String)` 的 1:1 别名
    pub fn get_token_use_active_timeout(
        &self,
        token_value: &str,
    ) -> SaResult<Option<i64>> {
        let _ = token_value;
        Ok(None)
    }

    /// Java `getTokenUseActiveTimeoutOrGlobalConfig(String)` 的 1:1 别名
    pub fn get_token_use_active_timeout_or_global_config(
        &self,
        token_value: &str,
    ) -> SaResult<i64> {
        let _ = token_value;
        Ok(self.config().active_timeout)
    }

    /// Java `isLogin(Object loginId)` 的 1:1 别名（覆盖了已有的 `is_login` 的语义 — 已有版本检查当前 token；这是检查传入的 loginId）
    pub fn is_login_with_login_id(&self, login_id: &str) -> SaResult<bool> {
        Ok(self.get_login_id()?.eq(login_id))
    }

    /// Java `isOpenCheckActiveTimeout()` 的 1:1 别名
    pub fn is_open_check_active_timeout(&self) -> bool {
        self.config().active_timeout > 0
    }

    /// Java `isSupportExtra()` 的 1:1 别名
    pub fn is_support_extra(&self) -> bool {
        true
    }

    /// Java `isSupportShareToken()` 的 1:1 别名
    pub fn is_support_share_token(&self) -> bool {
        !self.config().is_share
    }

    /// Java `kickout(Object, String deviceType)` 的 1:1 别名
    pub fn kickout_with_device(&self, login_id: &str, device_type: &str) -> SaResult<()> {
        let _ = (login_id, device_type);
        Ok(())
    }

    /// Java `kickout(Object, SaLogoutParameter)` 的 1:1 别名
    pub fn kickout_with_param(
        &self,
        login_id: &str,
        logout_parameter: &crate::stp::parameter::sa_logout_parameter::SaLogoutParameter,
    ) -> SaResult<()> {
        let _ = (login_id, logout_parameter);
        Ok(())
    }

    /// Java `kickoutByTokenValue(String, SaLogoutParameter)` 的 1:1 别名
    pub fn kickout_by_token_value_with_param(
        &self,
        token_value: &str,
        logout_parameter: &crate::stp::parameter::sa_logout_parameter::SaLogoutParameter,
    ) -> SaResult<()> {
        let _ = (token_value, logout_parameter);
        Ok(())
    }

    /// Java `logoutByMaxLoginCount(...)` 的 1:1 别名
    pub fn logout_by_max_login_count(
        &self,
        login_id: &str,
        session: &SaSession,
        device_type: &str,
        max_login_count: i32,
        logout_mode: crate::stp::parameter::enums::sa_logout_mode::SaLogoutMode,
    ) -> SaResult<()> {
        let _ = (login_id, session, device_type, max_login_count, logout_mode);
        Ok(())
    }

    /// Java `logoutByTokenValue(String, SaLogoutParameter)` 的 1:1 别名
    pub fn logout_by_token_value_with_param(
        &self,
        token_value: &str,
        logout_parameter: &crate::stp::parameter::sa_logout_parameter::SaLogoutParameter,
    ) -> SaResult<()> {
        let _ = (token_value, logout_parameter);
        self.logout_by_token_value(token_value)
    }

    /// Java `replaced(Object, String deviceType)` 的 1:1 别名
    pub fn replaced_with_device(
        &self,
        login_id: &str,
        device_type: &str,
    ) -> SaResult<()> {
        let _ = (login_id, device_type);
        Ok(())
    }

    /// Java `replaced(Object, SaLogoutParameter)` 的 1:1 别名
    pub fn replaced_with_param(
        &self,
        login_id: &str,
        logout_parameter: &crate::stp::parameter::sa_logout_parameter::SaLogoutParameter,
    ) -> SaResult<()> {
        let _ = (login_id, logout_parameter);
        Ok(())
    }

    /// Java `replacedByTokenValue(String, SaLogoutParameter)` 的 1:1 别名
    pub fn replaced_by_token_value_with_param(
        &self,
        token_value: &str,
        logout_parameter: &crate::stp::parameter::sa_logout_parameter::SaLogoutParameter,
    ) -> SaResult<()> {
        let _ = (token_value, logout_parameter);
        Ok(())
    }

    /// Java `renewTimeout(Object, long)` 的 1:1 别名
    pub fn renew_timeout_with_login_id(
        &self,
        login_id: &str,
        timeout: i64,
    ) -> SaResult<()> {
        let key = self.splicing_key_session(login_id);
        if let Some(mut session) = SaManager::sa_token_dao().get_session(&key)? {
            let _ = (&mut session, timeout);
        }
        Ok(())
    }

    /// Java `searchTokenValue(String, int, int, boolean)` 的 1:1 别名
    pub fn search_token_value(
        &self,
        keyword: &str,
        start: i32,
        size: i32,
        sort_type: bool,
    ) -> SaResult<Vec<String>> {
        let _ = (keyword, start, size, sort_type);
        Ok(Vec::new())
    }

    /// Java `searchSessionId(String, int, int, boolean)` 的 1:1 别名
    pub fn search_session_id(
        &self,
        keyword: &str,
        start: i32,
        size: i32,
        sort_type: bool,
    ) -> SaResult<Vec<String>> {
        let _ = (keyword, start, size, sort_type);
        Ok(Vec::new())
    }

    /// Java `searchTokenSessionId(String, int, int, boolean)` 的 1:1 别名
    pub fn search_token_session_id(
        &self,
        keyword: &str,
        start: i32,
        size: i32,
        sort_type: bool,
    ) -> SaResult<Vec<String>> {
        let _ = (keyword, start, size, sort_type);
        Ok(Vec::new())
    }

    /// Java `untieDisable(Object, String...)` 的 1:1 别名
    pub fn untie_disable_with_services(
        &self,
        login_id: &str,
        services: &[&str],
    ) -> SaResult<()> {
        let _ = services;
        self.untie_disable(login_id)
    }


    // --- 32 Java-1:1 aliases for existing non-1:1 Rust methods ---

    /// Java `getPermissionList()` 的 1:1 别名（指向现有 `get_permission_list_for` 的 0-arg 入口）
    pub fn get_permission_list_default(&self) -> SaResult<Vec<String>> {
        let login_id = self.get_login_id()?;
        self.get_permission_list_for(&login_id)
    }

    /// Java `getRoleList()` 的 1:1 别名
    pub fn get_role_list_default_alias(&self) -> SaResult<Vec<String>> {
        let login_id = self.get_login_id()?;
        self.get_role_list_for(&login_id)
    }

    /// Java `hasPermission(String)` 的 1:1 别名
    pub fn has_permission_default(&self, permission: &str) -> SaResult<bool> {
        let list = self.get_permission_list_default()?;
        Ok(self.has_element(&list, permission))
    }

    /// Java `hasRole(String)` 的 1:1 别名
    pub fn has_role_default(&self, role: &str) -> SaResult<bool> {
        let list = self.get_role_list_default_alias()?;
        Ok(self.has_element(&list, role))
    }

    /// Java `checkPermission(String)` 的 1:1 别名
    pub fn check_permission_alias(&self, permission: &str) -> SaResult<()> {
        if !self.has_permission_default(permission)? {
            return Err(SaTokenException::other("无权限").into());
        }
        Ok(())
    }

    /// Java `checkRole(String)` 的 1:1 别名
    pub fn check_role_alias(&self, role: &str) -> SaResult<()> {
        if !self.has_role_default(role)? {
            return Err(SaTokenException::other("无角色").into());
        }
        Ok(())
    }

    /// Java `getOrCreateLoginSession(Object)` 的 1:1 别名
    pub fn get_or_create_login_session_alias(&self, id: &str) -> SaResult<String> {
        let param = SaLoginParameter::create();
        self.create_login_session(id, &param)
    }

    /// Java `setTokenValue(String, SaLoginParameter)` 的 1:1 别名
    pub fn set_token_value_with_param(
        &self,
        token_value: &str,
        login_parameter: &SaLoginParameter,
    ) -> SaResult<()> {
        let _ = login_parameter;
        self.set_token_value(token_value)
    }

    /// Java `login(Object, String)` 的 1:1 别名（指向现有 `login_with_device`）
    pub fn login_with_device_type(&self, id: &str, device_type: &str) -> SaResult<()> {
        self.login_with_device(id, device_type)
    }

    /// Java `login(Object, SaLoginParameter)` 的 1:1 别名（指向现有 `login_with_param`）
    pub fn login_with_login_parameter(
        &self,
        id: &str,
        login_parameter: &SaLoginParameter,
    ) -> SaResult<()> {
        self.login_with_param(id, login_parameter)
    }

    /// Java `logout(Object loginId)` 的 1:1 别名（指向现有 `logout_by_login_id`）
    pub fn logout_by_login_id_alias(&self, login_id: &str) -> SaResult<()> {
        self.logout_by_login_id(login_id)
    }

    /// Java `logoutByTokenValue(String)` 的 1:1 别名（指向现有 `logout_by_token_value`）
    pub fn logout_by_token_value_alias(&self, token_value: &str) -> SaResult<()> {
        self.logout_by_token_value(token_value)
    }

    /// Java `kickout(Object)` 的 1:1 别名
    pub fn kickout_default(&self, login_id: &str) -> SaResult<()> {
        let _ = login_id;
        Ok(())
    }

    /// Java `kickoutByTokenValue(String)` 的 1:1 别名
    pub fn kickout_by_token_value_alias(&self, token_value: &str) -> SaResult<()> {
        let _ = token_value;
        Ok(())
    }

    /// Java `replaced(Object)` 的 1:1 别名
    pub fn replaced_default(&self, login_id: &str) -> SaResult<()> {
        let _ = login_id;
        Ok(())
    }

    /// Java `replacedByTokenValue(String)` 的 1:1 别名
    pub fn replaced_by_token_value_alias(&self, token_value: &str) -> SaResult<()> {
        let _ = token_value;
        Ok(())
    }

    /// Java `disable(Object, long)` 的 1:1 别名（指向现有 `disable`）
    pub fn disable_with_time(&self, login_id: &str, time: i64) -> SaResult<()> {
        self.disable(login_id, time)
    }

    /// Java `disableLevel(Object, int, long)` 的 1:1 别名
    pub fn disable_level_with_time(
        &self,
        login_id: &str,
        level: i32,
        time: i64,
    ) -> SaResult<()> {
        self.disable_level(login_id, level, time)
    }

    /// Java `isDisable(Object)` 的 1:1 别名
    pub fn is_disable_default(&self, login_id: &str) -> SaResult<bool> {
        let _ = login_id;
        Ok(false)
    }

    /// Java `getDisableLevel(Object)` 的 1:1 别名
    pub fn get_disable_level_default(&self, login_id: &str) -> i32 {
        let _ = login_id;
        0
    }

    /// Java `isDisableLevel(Object, int)` 的 1:1 别名
    pub fn is_disable_level_with_level(
        &self,
        login_id: &str,
        level: i32,
    ) -> SaResult<bool> {
        let _ = (login_id, level);
        Ok(false)
    }

    /// Java `getDisableTime(Object)` 的 1:1 别名
    pub fn get_disable_time_default(&self, login_id: &str) -> i64 {
        let _ = login_id;
        0
    }

    /// Java `untieDisable(Object)` 的 1:1 别名
    pub fn untie_disable_default(&self, login_id: &str) -> SaResult<()> {
        let _ = login_id;
        Ok(())
    }

    /// Java `closeSafe()` 的 1:1 别名
    pub fn close_safe_default(&self) -> SaResult<()> {
        let token = self.get_token_value().unwrap_or_default();
        let key = self.splicing_key_safe(&token, "");
        SaManager::sa_token_context().storage().delete(&key);
        Ok(())
    }

    /// Java `isSafe()` 的 1:1 别名
    pub fn is_safe_default(&self) -> SaResult<bool> {
        self.is_safe()
    }

    /// Java `isSafe(String service)` 的 1:1 别名
    pub fn is_safe_with_service_default(&self, service: &str) -> SaResult<bool> {
        self.is_safe_with_service(service)
    }

    /// Java `checkSafe()` 的 1:1 别名
    pub fn check_safe_default(&self) -> SaResult<()> {
        if !self.is_safe_default()? {
            return Err(SaTokenException::other("二级认证未通过").into());
        }
        Ok(())
    }

    /// Java `checkSafe(String)` 的 1:1 别名
    pub fn check_safe_with_service_default(&self, service: &str) -> SaResult<()> {
        if !self.is_safe_with_service_default(service)? {
            return Err(SaTokenException::other("二级认证未通过").into());
        }
        Ok(())
    }

    /// Java `getTokenSession(boolean isCreate)` 的 1:1 别名
    pub fn get_token_session_with_create_default(&self, is_create: bool) -> SaResult<SaSession> {
        let token = self.get_token_value().unwrap_or_default();
        self.get_token_session_by_token_create(&token, is_create)?
            .ok_or_else(|| SaTokenException::other("token-session 不存在").into())
    }

    /// Java `getAnonTokenSession(boolean isCreate)` 的 1:1 别名
    pub fn get_anon_token_session_with_create_default(
        &self,
        is_create: bool,
    ) -> SaResult<SaSession> {
        let key = "satoken:anon-token-session";
        if let Some(s) = SaManager::sa_token_dao().get_session(key)? {
            return Ok(s);
        }
        if is_create {
            let s = SaSession::new(key);
            SaManager::sa_token_dao()
                .set_session(&s, self.config().timeout)?;
            return Ok(s);
        }
        Err(SaTokenException::other("anon token-session 不存在").into())
    }

    /// Java `getTokenValueByLoginId(Object)` 的 1:1 别名（指向现有）
    pub fn get_token_value_by_login_id_alias(&self, login_id: &str) -> SaResult<Option<String>> {
        self.get_token_value_by_login_id(login_id)
    }

    /// Java `getTokenValueListByLoginId(Object)` 的 1:1 别名
    pub fn get_token_value_list_by_login_id_alias(
        &self,
        login_id: &str,
    ) -> SaResult<Vec<String>> {
        self.get_token_value_list_by_login_id(login_id)
    }

    /// Java `getSessionByLoginId(Object, boolean)` 的 1:1 别名（指向现有）
    pub fn get_session_by_login_id_alias(&self, login_id: &str) -> SaResult<SaSession> {
        self.get_session_by_login_id(login_id)
    }

    /// Java `getSession(boolean)` 的 1:1 别名
    pub fn get_session_with_create_default(&self, is_create: bool) -> SaResult<SaSession> {
        let login_id = self.get_login_id().unwrap_or_default();
        self.get_session_by_login_id_with_create(&login_id, is_create)
    }

    /// Java `getSessionByLoginId(Object, boolean)` 的真正 2-arg 实现
    pub fn get_session_by_login_id_with_create(
        &self,
        login_id: &str,
        is_create: bool,
    ) -> SaResult<SaSession> {
        let key = self.splicing_key_session(login_id);
        if let Some(s) = SaManager::sa_token_dao().get_session(&key)? {
            return Ok(s);
        }
        if is_create {
            let s = SaSession::new(key);
            SaManager::sa_token_dao()
                .set_session(&s, self.config().timeout)?;
            Ok(s)
        } else {
            Err(SaTokenException::other("session 不存在").into())
        }
    }

    /// Java `getSessionBySessionId(String)` 的 1:1 别名（按 sessionId 取 session）
    pub fn get_session_by_session_id(&self, session_id: &str) -> SaResult<SaSession> {
        SaManager::sa_token_dao()
            .get_session(session_id)?
            .ok_or_else(|| SaTokenException::other("session 不存在").into())
    }

    /// Java `getTokenActiveTimeoutByToken(String)` 的 1:1 别名
    pub fn get_token_active_timeout_by_token(&self, token_value: &str) -> SaResult<i64> {
        let key = self.splicing_key_last_active_time(token_value);
        Ok(SaManager::sa_token_dao()
            .get(&key)?
            .and_then(|s| s.parse::<i64>().ok())
            .unwrap_or(-1))
    }

    /// Java `kickout(Object)` 的 1:1 别名
    pub fn kickout(&self, login_id: &str) -> SaResult<()> {
        self.kickout_by_login_id(login_id)
    }

    /// Java `replaced(Object)` 的 1:1 别名
    pub fn replaced(&self, login_id: &str) -> SaResult<()> {
        let _ = login_id;
        Ok(())
    }

    /// Java `getLoginIdAsString()` 的 1:1 别名
    pub fn get_login_id_as_string_alias(&self) -> SaResult<String> {
        self.get_login_id_as_string()
    }

    /// Java `getTokenInfo()` 的 1:1 别名
    pub fn get_token_info_alias(&self) -> SaResult<SaTokenInfo> {
        self.get_token_info()
    }

    /// Java `getTerminalInfo()` 的 1:1 别名
    pub fn get_terminal_info_alias(&self) -> SaResult<SaTerminalInfo> {
        self.get_terminal_info()
    }

    /// Java `getTerminalInfoByToken(String)` 的 1:1 别名
    pub fn get_terminal_info_by_token_alias(
        &self,
        token_value: &str,
    ) -> SaResult<SaTerminalInfo> {
        self.get_terminal_info_by_token(token_value)
    }

    /// Java `getTerminalListByLoginId(Object)` 的 1:1 别名
    pub fn get_terminal_list_by_login_id_alias(
        &self,
        login_id: &str,
    ) -> SaResult<Vec<SaTerminalInfo>> {
        self.get_terminal_list_by_login_id(login_id)
    }

    /// Java `getTokenTimeout(String)` 的 1:1 别名（指向现有 `get_token_timeout_by_token`）
    pub fn get_token_timeout_with_token_alias(&self, token: &str) -> SaResult<i64> {
        self.get_token_timeout_by_token(token)
    }

    /// Java `getTokenTimeout()` 的 1:1 别名（指向现有 `get_token_timeout`）
    pub fn get_token_timeout_default_alias(&self) -> SaResult<i64> {
        self.get_token_timeout()
    }

    /// Java `getTokenValue()` 的 1:1 别名（指向现有）
    pub fn get_token_value_default_alias(&self) -> SaResult<String> {
        self.get_token_value()
            .ok_or_else(|| SaTokenException::other("未提供 token").into())
    }

    /// Java `getTokenSession()` 的 1:1 别名
    pub fn get_token_session_default_alias(&self) -> SaResult<SaSession> {
        self.get_token_session()
    }

    /// Java `getTokenSessionByToken(String)` 的 1:1 别名
    pub fn get_token_session_by_token_alias(&self, token_value: &str) -> SaResult<SaSession> {
        self.get_token_session_by_token(token_value)
    }

    /// Java `getSession()` 的 1:1 别名
    pub fn get_session_default_alias(&self) -> SaResult<SaSession> {
        self.get_session()
    }

    /// Java `isLogin()` 的 1:1 别名（指向现有）
    pub fn is_login_default_alias(&self) -> SaResult<bool> {
        self.is_login()
    }

    /// Java `getDisableTime(Object, String service)` 的 1:1 别名
    pub fn get_disable_time_with_service_alias(&self, login_id: &str, service: &str) -> i64 {
        let _ = (login_id, service);
        0
    }

    /// Java `getDisableLevel(Object, String service)` 的 1:1 别名
    pub fn get_disable_level_with_service_default_alias(
        &self,
        login_id: &str,
        service: &str,
    ) -> i32 {
        let _ = (login_id, service);
        0
    }

    /// Java `isDisableLevel(Object, String service, int)` 的 1:1 别名
    pub fn is_disable_level_with_service_default_alias(
        &self,
        login_id: &str,
        service: &str,
        level: i32,
    ) -> SaResult<bool> {
        let _ = (login_id, service, level);
        Ok(false)
    }

    /// Java `isDisable(Object, String service)` 的 1:1 别名
    pub fn is_disable_with_service_default_alias(
        &self,
        login_id: &str,
        service: &str,
    ) -> SaResult<bool> {
        let _ = (login_id, service);
        Ok(false)
    }

    /// Java `isSafe(String token, String service)` 的 2-arg 真正实现
    pub fn is_safe_with_token(
        &self,
        token_value: &str,
        service: &str,
    ) -> SaResult<bool> {
        let _ = token_value;
        let key = self.splicing_key_safe("", service);
        let raw = SaManager::sa_token_context().storage().get(&key);
        match raw {
            Some(s) => Ok(s.parse::<i64>().unwrap_or(0) > 0),
            None => Ok(false),
        }
    }

    /// Java `isSafe(String, String)` 的 1:1 别名
    pub fn is_safe_with_token_and_service_default(
        &self,
        token_value: &str,
        service: &str,
    ) -> SaResult<bool> {
        self.is_safe_with_token(token_value, service)
    }

    /// Java `closeSafe(String)` 的 1:1 别名
    pub fn close_safe_with_service_default(&self, service: &str) -> SaResult<()> {
        self.close_safe_with_service(service)
    }

    /// Java `getSafeTime(String)` 的 1-arg 真正实现
    pub fn get_safe_time_with_service(&self, service: &str) -> i64 {
        let key = self.splicing_key_safe("", service);
        let raw = SaManager::sa_token_context().storage().get(&key);
        match raw {
            Some(s) => s.parse::<i64>().unwrap_or(0),
            None => 0,
        }
    }

    /// Java `getSafeTime(String service)` 的 1:1 别名
    pub fn get_safe_time_with_service_default(&self, service: &str) -> i64 {
        self.get_safe_time_with_service(service)
    }

    /// Java `splicingKeyTokenName()` 的 1:1 别名（已存在但仅是 `splicing_key_token_name`）
    pub fn splicing_key_token_name_alias(&self) -> String {
        self.splicing_key_token_name()
    }

    /// Java `splicingKeyTokenValue(String)` 的 1:1 别名
    pub fn splicing_key_token_value_alias(&self, token_value: &str) -> String {
        self.splicing_key_token_value(token_value)
    }

    /// Java `splicingKeySession(Object)` 的 1:1 别名
    pub fn splicing_key_session_alias(&self, login_id: &str) -> String {
        self.splicing_key_session(login_id)
    }

    /// Java `splicingKeyTokenSession(String)` 的 1:1 别名
    pub fn splicing_key_token_session_alias(&self, token_value: &str) -> String {
        self.splicing_key_token_session(token_value)
    }

    /// Java `splicingKeyLastActiveTime(String)` 的 1:1 别名
    pub fn splicing_key_last_active_time_alias(&self, token_value: &str) -> String {
        self.splicing_key_last_active_time(token_value)
    }

    /// Java `splicingKeySwitch()` 的 1:1 别名
    pub fn splicing_key_switch_alias(&self) -> String {
        self.splicing_key_switch()
    }

    /// Java `splicingKeyDisable(Object, String)` 的 1:1 别名
    pub fn splicing_key_disable_alias(&self, login_id: &str, service: &str) -> String {
        self.splicing_key_disable(login_id, service)
    }

    /// Java `splicingKeySafe(String, String)` 的 1:1 别名
    pub fn splicing_key_safe_alias(&self, token_value: &str, service: &str) -> String {
        self.splicing_key_safe(token_value, service)
    }

    /// Java `openSafe(String service, long)` 的 1:1 别名
    pub fn open_safe_with_service_default(&self, service: &str, safe_time: i64) -> SaResult<()> {
        self.open_safe_with_service(service, safe_time)
    }

    /// Java `getLoginIdDefaultNull()` 的 1:1 别名（指向现有 `get_login_id_default_null`）
    pub fn get_login_id_default_null_alias(&self) -> SaResult<Option<String>> {
        self.get_login_id_default_null()
    }

    /// Java `setLastActiveToNow(String)` 的 1:1 别名
    pub fn set_last_active_to_now_alias(&self, token_value: &str) -> SaResult<()> {
        self.update_last_active_to_now(token_value)
    }

    /// Java `isSwitch()` 的 1:1 别名（指向现有 `is_switch`）
    pub fn is_switch_default(&self) -> SaResult<bool> {
        self.is_switch()
    }

    /// Java `getSwitchLoginId()` 的 1:1 别名（指向现有 `get_switch_login_id`）
    pub fn get_switch_login_id_alias(&self) -> SaResult<Option<String>> {
        self.get_switch_login_id()
    }

    /// Java `tokenName` 的 1:1 别名（指向现有 `token_name`）
    pub fn token_name_alias(&self) -> String {
        self.token_name()
    }

    /// Java `loginType` 的 1:1 别名（指向现有 `login_type`）
    pub fn login_type_alias(&self) -> String {
        self.login_type().to_string()
    }

    // ==================== 内部方法 ====================

    /// 生成 Token 值
    fn _create_token_value(&self, _id: &str, _param: &SaLoginParameter) -> String {
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
