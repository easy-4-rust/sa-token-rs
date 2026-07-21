//! 核心逻辑（对应 Java `cn.dev33.satoken.stp.StpLogic`）。
use std::sync::Arc;

use crate::config::sa_token_config::SaTokenConfig;
use crate::exception::{SaResult, SaTokenException};
use crate::manager::SaManager;
use crate::session::sa_session::SaSession;
use crate::session::sa_terminal_info::SaTerminalInfo;
use crate::stp::parameter::sa_login_parameter::SaLoginParameter;
use crate::stp::sa_token_info::SaTokenInfo;
use crate::util::sa_fox_util;

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
    pub fn create_login_session(
        &self,
        id: &str,
        param: &SaLoginParameter,
    ) -> SaResult<String> {
        let config = self.config();

        // 生成 Token 值
        let token_value = self.create_token_value(id, param);

        // 获取或创建 Session
        let session = self.get_or_create_login_session(id)?;

        // 添加终端信息
        let mut terminal = SaTerminalInfo::new(
            session.terminal_list().len() as i32 + 1,
            &token_value,
            &param.device_type,
        );
        terminal.set_device_id(&param.device_id);
        if let Some(ref extra) = param.terminal_extra_data {
            terminal.set_extra("extra", extra.clone());
        }

        // 更新 Session
        {
            let mut session = session;
            session.add_terminal(terminal);

            // 保存 Session 到 DAO
            let dao = SaManager::sa_token_dao();
            let timeout = param.get_timeout(&config);
            let session_key = self.splicing_key_session(id);
            if dao.get_session(&session_key).is_some() {
                dao.update_session(&session);
            } else {
                dao.set_session(&session, timeout);
            }
        }

        // 保存 Token-LoginId 映射
        let dao = SaManager::sa_token_dao();
        let timeout = param.get_timeout(&config);
        let token_key = self.splicing_key_token_value(&token_value);
        dao.set(&token_key, id, timeout);

        // 保存最后活跃时间
        self.set_last_active_to_now(&token_value)?;

        Ok(token_value)
    }

    /// 获取或创建登录会话
    pub fn get_or_create_login_session(&self, id: &str) -> SaResult<SaSession> {
        let session_key = self.splicing_key_session(id);
        let dao = SaManager::sa_token_dao();

        if let Some(session) = dao.get_session(&session_key) {
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
        if let Some(login_id) = self.get_login_id_default_null() {
            self.logout_by_login_id(&login_id)?;
        }
        Ok(())
    }

    /// 按 loginId 注销
    pub fn logout_by_login_id(&self, login_id: &str) -> SaResult<()> {
        // 获取该账号的所有 Token
        let token_list = self.get_token_value_list_by_login_id(login_id);

        // 逐个注销
        for token in token_list {
            self.logout_by_token_value(&token)?;
        }

        // 删除 Session
        let session_key = self.splicing_key_session(login_id);
        SaManager::sa_token_dao().delete_session(&session_key);

        Ok(())
    }

    /// 按 Token 注销
    pub fn logout_by_token_value(&self, token_value: &str) -> SaResult<()> {
        // 获取 loginId
        let login_id = self.get_login_id_by_token(token_value);
        if login_id.is_none() {
            return Ok(());
        }
        let login_id = login_id.unwrap();

        // 删除 Token-LoginId 映射
        let token_key = self.splicing_key_token_value(token_value);
        SaManager::sa_token_dao().delete(&token_key);

        // 删除最后活跃时间
        let active_key = self.splicing_key_last_active_time(token_value);
        SaManager::sa_token_dao().delete(&active_key);

        // 从 Session 中移除终端
        let session_key = self.splicing_key_session(&login_id);
        if let Some(mut session) = SaManager::sa_token_dao().get_session(&session_key) {
            session.remove_terminal(token_value);
            SaManager::sa_token_dao().update_session(&session);
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
        let token_list = self.get_token_value_list_by_login_id(login_id);
        for token in token_list {
            self.kickout_by_token_value(&token)?;
        }
        Ok(())
    }

    /// 按 Token 踢人下线
    pub fn kickout_by_token_value(&self, token_value: &str) -> SaResult<()> {
        let login_id = self.get_login_id_by_token(token_value);
        if login_id.is_none() {
            return Ok(());
        }
        let login_id = login_id.unwrap();

        // 删除 Token
        let token_key = self.splicing_key_token_value(token_value);
        SaManager::sa_token_dao().delete(&token_key);

        // 删除最后活跃时间
        let active_key = self.splicing_key_last_active_time(token_value);
        SaManager::sa_token_dao().delete(&active_key);

        // 从 Session 中移除终端
        let session_key = self.splicing_key_session(&login_id);
        if let Some(mut session) = SaManager::sa_token_dao().get_session(&session_key) {
            session.remove_terminal(token_value);
            SaManager::sa_token_dao().update_session(&session);
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
        let token_list = self.get_token_value_list_by_login_id(login_id);
        for token in token_list {
            self.replaced_by_token_value(&token)?;
        }
        Ok(())
    }

    /// 按 Token 顶人下线
    pub fn replaced_by_token_value(&self, token_value: &str) -> SaResult<()> {
        let login_id = self.get_login_id_by_token(token_value);
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
        SaManager::sa_token_dao().delete(&token_key);

        // 删除最后活跃时间
        let active_key = self.splicing_key_last_active_time(token_value);
        SaManager::sa_token_dao().delete(&active_key);

        Ok(())
    }

    // ==================== 登录状态 ====================

    /// 是否已登录
    pub fn is_login(&self) -> bool {
        self.get_login_id_default_null().is_some()
    }

    /// 检查是否已登录（未登录抛异常）
    pub fn check_login(&self) -> SaResult<()> {
        if !self.is_login() {
            return Err(SaTokenException::not_login(
                "未登录",
                &self.login_type,
            ));
        }
        Ok(())
    }

    /// 获取当前登录 ID
    pub fn get_login_id(&self) -> SaResult<String> {
        self.get_login_id_default_null().ok_or_else(|| {
            SaTokenException::not_login("未登录", &self.login_type)
        })
    }

    /// 获取当前登录 ID（未登录返回 None）
    pub fn get_login_id_default_null(&self) -> Option<String> {
        let token_value = self.get_token_value()?;
        self.get_login_id_by_token(&token_value)
    }

    /// 获取当前登录 ID（转为 String）
    pub fn get_login_id_as_string(&self) -> SaResult<String> {
        self.get_login_id()
    }

    /// 获取当前登录 ID（转为 i32）
    pub fn get_login_id_as_i32(&self) -> SaResult<i32> {
        let id = self.get_login_id()?;
        id.parse::<i32>().map_err(|_| {
            SaTokenException::other(format!("无法将 loginId 转换为 i32: {}", id))
        })
    }

    /// 获取当前登录 ID（转为 i64）
    pub fn get_login_id_as_i64(&self) -> SaResult<i64> {
        let id = self.get_login_id()?;
        id.parse::<i64>().map_err(|_| {
            SaTokenException::other(format!("无法将 loginId 转换为 i64: {}", id))
        })
    }

    /// 根据 Token 获取 loginId
    pub fn get_login_id_by_token(&self, token_value: &str) -> Option<String> {
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
        let token_value = self.get_token_value().ok_or_else(|| {
            SaTokenException::not_login("未提供 Token", &self.login_type)
        })?;

        let login_id = self.get_login_id_by_token(&token_value).ok_or_else(|| {
            SaTokenException::not_login("Token 无效", &self.login_type)
        })?;

        let mut info = SaTokenInfo::new(self.token_name(), &token_value);
        info.login_id = login_id;
        info.token_timeout = self.get_token_timeout_by_token(&token_value);
        info.login_device_type = self
            .get_login_device_type_by_token(&token_value)
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
            let cookie = crate::context::model::sa_cookie::SaCookie::new(
                &config.token_name,
                token_value,
            );
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
            .get_session(&session_key)
            .ok_or_else(|| SaTokenException::other("Session 不存在"))
    }

    /// 获取 Token-Session
    pub fn get_token_session(&self) -> SaResult<SaSession> {
        let token_value = self.get_token_value().ok_or_else(|| {
            SaTokenException::not_login("未提供 Token", &self.login_type)
        })?;
        self.get_token_session_by_token(&token_value)
    }

    /// 按 Token 获取 Token-Session
    pub fn get_token_session_by_token(&self, token_value: &str) -> SaResult<SaSession> {
        let session_key = self.splicing_key_token_session(token_value);
        SaManager::sa_token_dao()
            .get_session(&session_key)
            .ok_or_else(|| SaTokenException::other("Token-Session 不存在"))
    }

    // ==================== Token 超时 ====================

    /// 获取当前 Token 超时时间
    pub fn get_token_timeout(&self) -> i64 {
        if let Some(token_value) = self.get_token_value() {
            self.get_token_timeout_by_token(&token_value)
        } else {
            -2
        }
    }

    /// 按 Token 获取超时时间
    pub fn get_token_timeout_by_token(&self, token_value: &str) -> i64 {
        let token_key = self.splicing_key_token_value(token_value);
        SaManager::sa_token_dao().get_timeout(&token_key)
    }

    /// 续签当前 Token
    pub fn renew_timeout(&self, timeout: i64) -> SaResult<()> {
        let token_value = self.get_token_value().ok_or_else(|| {
            SaTokenException::not_login("未提供 Token", &self.login_type)
        })?;
        self.renew_timeout_by_token(&token_value, timeout)
    }

    /// 续签指定 Token
    pub fn renew_timeout_by_token(&self, token_value: &str, timeout: i64) -> SaResult<()> {
        let token_key = self.splicing_key_token_value(token_value);
        SaManager::sa_token_dao().update_timeout(&token_key, timeout);

        // 更新最后活跃时间
        self.set_last_active_to_now(token_value)?;

        Ok(())
    }

    // ==================== 活跃超时 ====================

    /// 更新最后活跃时间为当前
    pub fn set_last_active_to_now(&self, token_value: &str) -> SaResult<()> {
        let active_key = self.splicing_key_last_active_time(token_value);
        let now = sa_fox_util::now_timestamp().to_string();
        SaManager::sa_token_dao().set(&active_key, &now, -1);
        Ok(())
    }

    /// 获取 Token 最后活跃时间
    pub fn get_token_last_active_time(&self, token_value: &str) -> i64 {
        let active_key = self.splicing_key_last_active_time(token_value);
        SaManager::sa_token_dao()
            .get(&active_key)
            .and_then(|s| s.parse().ok())
            .unwrap_or(0)
    }

    /// 检查活跃超时
    pub fn check_active_timeout(&self) -> SaResult<()> {
        let config = self.config();
        if config.active_timeout <= 0 {
            return Ok(());
        }

        let token_value = self.get_token_value().ok_or_else(|| {
            SaTokenException::not_login("未提供 Token", &self.login_type)
        })?;

        let last_active = self.get_token_last_active_time(&token_value);
        let now = sa_fox_util::now_timestamp();

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
        let token_value = self.get_token_value().ok_or_else(|| {
            SaTokenException::not_login("未提供 Token", &self.login_type)
        })?;
        self.get_login_device_type_by_token(&token_value)
            .ok_or_else(|| SaTokenException::other("无法获取设备类型"))
    }

    /// 按 Token 获取设备类型
    pub fn get_login_device_type_by_token(&self, token_value: &str) -> Option<String> {
        let login_id = self.get_login_id_by_token(token_value)?;
        let session_key = self.splicing_key_session(&login_id);
        let session = SaManager::sa_token_dao().get_session(&session_key)?;
        session
            .get_terminal(token_value)
            .map(|t| t.device_type().to_string())
    }

    /// 获取当前登录设备 ID
    pub fn get_login_device_id(&self) -> SaResult<String> {
        let token_value = self.get_token_value().ok_or_else(|| {
            SaTokenException::not_login("未提供 Token", &self.login_type)
        })?;
        self.get_login_device_id_by_token(&token_value)
            .ok_or_else(|| SaTokenException::other("无法获取设备 ID"))
    }

    /// 按 Token 获取设备 ID
    pub fn get_login_device_id_by_token(&self, token_value: &str) -> Option<String> {
        let login_id = self.get_login_id_by_token(token_value)?;
        let session_key = self.splicing_key_session(&login_id);
        let session = SaManager::sa_token_dao().get_session(&session_key)?;
        session
            .get_terminal(token_value)
            .map(|t| t.device_id().to_string())
    }

    // ==================== 终端查询 ====================

    /// 获取指定账号的终端列表
    pub fn get_terminal_list_by_login_id(&self, login_id: &str) -> SaResult<Vec<SaTerminalInfo>> {
        let session_key = self.splicing_key_session(login_id);
        let session = SaManager::sa_token_dao()
            .get_session(&session_key)
            .ok_or_else(|| SaTokenException::other("Session 不存在"))?;
        Ok(session.terminal_list().to_vec())
    }

    /// 获取当前终端信息
    pub fn get_terminal_info(&self) -> SaResult<SaTerminalInfo> {
        let token_value = self.get_token_value().ok_or_else(|| {
            SaTokenException::not_login("未提供 Token", &self.login_type)
        })?;
        self.get_terminal_info_by_token(&token_value)
    }

    /// 按 Token 获取终端信息
    pub fn get_terminal_info_by_token(&self, token_value: &str) -> SaResult<SaTerminalInfo> {
        let login_id = self.get_login_id_by_token(token_value).ok_or_else(|| {
            SaTokenException::not_login("Token 无效", &self.login_type)
        })?;
        let session_key = self.splicing_key_session(&login_id);
        let session = SaManager::sa_token_dao()
            .get_session(&session_key)
            .ok_or_else(|| SaTokenException::other("Session 不存在"))?;
        session
            .get_terminal(token_value)
            .cloned()
            .ok_or_else(|| SaTokenException::other("终端不存在"))
    }

    /// 获取指定账号的 Token 列表
    pub fn get_token_value_list_by_login_id(&self, login_id: &str) -> Vec<String> {
        let session_key = self.splicing_key_session(login_id);
        if let Some(session) = SaManager::sa_token_dao().get_session(&session_key) {
            session
                .terminal_list()
                .iter()
                .map(|t| t.token_value().to_string())
                .collect()
        } else {
            Vec::new()
        }
    }

    /// 获取指定账号的 Token 值
    pub fn get_token_value_by_login_id(&self, login_id: &str) -> Option<String> {
        let list = self.get_token_value_list_by_login_id(login_id);
        list.into_iter().next()
    }

    // ==================== Key 拼接 ====================

    /// 拼接 Token 名称 Key
    pub fn splicing_key_token_name(&self) -> String {
        format!("satoken:login:token-name:{}", self.login_type)
    }

    /// 拼接 Token 值 Key
    pub fn splicing_key_token_value(&self, token_value: &str) -> String {
        format!("satoken:login:token:{}", token_value)
    }

    /// 拼接 Session Key
    pub fn splicing_key_session(&self, login_id: &str) -> String {
        format!("satoken:login:session:{}:{}", self.login_type, login_id)
    }

    /// 拼接 Token-Session Key
    pub fn splicing_key_token_session(&self, token_value: &str) -> String {
        format!("satoken:login:token-session:{}", token_value)
    }

    /// 拼接最后活跃时间 Key
    pub fn splicing_key_last_active_time(&self, token_value: &str) -> String {
        format!("satoken:login:last-active:{}", token_value)
    }

    /// 拼接封禁 Key
    pub fn splicing_key_disable(&self, login_id: &str, service: &str) -> String {
        if service.is_empty() {
            format!("satoken:disable:{}:{}", self.login_type, login_id)
        } else {
            format!("satoken:disable:{}:{}:{}", self.login_type, login_id, service)
        }
    }

    /// 拼接二级认证 Key
    pub fn splicing_key_safe(&self, token_value: &str, service: &str) -> String {
        if service.is_empty() {
            format!("satoken:safe:{}", token_value)
        } else {
            format!("satoken:safe:{}:{}", token_value, service)
        }
    }

    /// 拼接切换账号 Key
    pub fn splicing_key_switch(&self) -> String {
        format!("satoken:switch:{}", self.login_type)
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
            return Err(SaTokenException::not_role(roles.join(","), &self.login_type));
        }
        Ok(())
    }

    /// 检查当前账号是否具有指定角色（OR 模式）
    pub fn check_role_or(&self, roles: &[&str]) -> SaResult<()> {
        if !self.has_role_or(roles)? {
            return Err(SaTokenException::not_role(roles.join(","), &self.login_type));
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
        Ok(permissions.iter().all(|p| permission_list.contains(&p.to_string())))
    }

    /// 当前账号是否具有指定权限（OR 模式）
    pub fn has_permission_or(&self, permissions: &[&str]) -> SaResult<bool> {
        let login_id = self.get_login_id()?;
        let permission_list = self.get_permission_list_for(&login_id)?;
        Ok(permissions.iter().any(|p| permission_list.contains(&p.to_string())))
    }

    /// 检查当前账号是否具有指定权限（不满足则抛异常）
    pub fn check_permission(&self, permission: &str) -> SaResult<()> {
        if !self.has_permission(permission)? {
            return Err(SaTokenException::not_permission(permission, &self.login_type));
        }
        Ok(())
    }

    /// 检查当前账号是否具有指定权限（AND 模式）
    pub fn check_permission_and(&self, permissions: &[&str]) -> SaResult<()> {
        if !self.has_permission_and(permissions)? {
            return Err(SaTokenException::not_permission(permissions.join(","), &self.login_type));
        }
        Ok(())
    }

    /// 检查当前账号是否具有指定权限（OR 模式）
    pub fn check_permission_or(&self, permissions: &[&str]) -> SaResult<()> {
        if !self.has_permission_or(permissions)? {
            return Err(SaTokenException::not_permission(permissions.join(","), &self.login_type));
        }
        Ok(())
    }

    // ==================== 禁用 ====================

    /// 封禁账号
    pub fn disable(&self, login_id: &str, time: i64) -> SaResult<()> {
        let key = self.splicing_key_disable(login_id, "");
        SaManager::sa_token_dao().set(&key, "1", time);
        Ok(())
    }

    /// 解封账号
    pub fn untie_disable(&self, login_id: &str) -> SaResult<()> {
        let key = self.splicing_key_disable(login_id, "");
        SaManager::sa_token_dao().delete(&key);
        Ok(())
    }

    /// 获取封禁剩余时间
    pub fn get_disable_time(&self, login_id: &str) -> i64 {
        let key = self.splicing_key_disable(login_id, "");
        SaManager::sa_token_dao().get_timeout(&key)
    }

    /// 指定账号是否被封禁
    pub fn is_disable(&self, login_id: &str) -> bool {
        let key = self.splicing_key_disable(login_id, "");
        SaManager::sa_token_dao().get(&key).is_some()
    }

    /// 检查账号是否被封禁（被封禁则抛异常）
    pub fn check_disable(&self, login_id: &str) -> SaResult<()> {
        if self.is_disable(login_id) {
            return Err(SaTokenException::disable_service(login_id, "", self.get_disable_time(login_id)));
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
        let token_value = self.get_token_value().ok_or_else(|| {
            SaTokenException::not_login("未提供 Token", &self.login_type)
        })?;
        let key = self.splicing_key_safe(&token_value, service);
        SaManager::sa_token_dao().set(&key, "1", safe_time);
        Ok(())
    }

    /// 当前是否处于二级认证
    pub fn is_safe(&self) -> bool {
        self.is_safe_with_service("")
    }

    /// 指定业务是否处于二级认证
    pub fn is_safe_with_service(&self, service: &str) -> bool {
        if let Some(token_value) = self.get_token_value() {
            let key = self.splicing_key_safe(&token_value, service);
            SaManager::sa_token_dao().get(&key).is_some()
        } else {
            false
        }
    }

    /// 校验二级认证（未通过则抛异常）
    pub fn check_safe(&self) -> SaResult<()> {
        self.check_safe_with_service("")
    }

    /// 校验指定业务的二级认证
    pub fn check_safe_with_service(&self, service: &str) -> SaResult<()> {
        if !self.is_safe_with_service(service) {
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
            SaManager::sa_token_dao().delete(&key);
        }
        Ok(())
    }

    // ==================== 切换账号 ====================

    /// 临时切换到指定账号身份
    pub fn switch_to(&self, login_id: &str) -> SaResult<()> {
        let _token_value = self.get_token_value().ok_or_else(|| {
            SaTokenException::not_login("未提供 Token", &self.login_type)
        })?;
        let key = self.splicing_key_switch();
        SaManager::sa_token_dao().set(&key, login_id, -1);
        Ok(())
    }

    /// 结束切换
    pub fn end_switch(&self) -> SaResult<()> {
        let key = self.splicing_key_switch();
        SaManager::sa_token_dao().delete(&key);
        Ok(())
    }

    /// 当前是否处于切换状态
    pub fn is_switch(&self) -> bool {
        let key = self.splicing_key_switch();
        SaManager::sa_token_dao().get(&key).is_some()
    }

    /// 获取临时切换的 loginId
    pub fn get_switch_login_id(&self) -> Option<String> {
        let key = self.splicing_key_switch();
        SaManager::sa_token_dao().get(&key)
    }

    // ==================== 内部方法 ====================

    /// 生成 Token 值
    fn create_token_value(&self, _id: &str, _param: &SaLoginParameter) -> String {
        let config = self.config();
        match config.token_style {
            crate::config::sa_token_config::SaTokenStyle::Uuid => {
                uuid::Uuid::new_v4().to_string()
            }
            crate::config::sa_token_config::SaTokenStyle::SimpleUuid => {
                uuid::Uuid::new_v4()
                    .to_string()
                    .replace('-', "")
            }
            crate::config::sa_token_config::SaTokenStyle::Random32 => {
                sa_fox_util::random_string(32)
            }
            crate::config::sa_token_config::SaTokenStyle::Random64 => {
                sa_fox_util::random_string(64)
            }
            crate::config::sa_token_config::SaTokenStyle::Random128 => {
                sa_fox_util::random_string(128)
            }
            crate::config::sa_token_config::SaTokenStyle::Base64 => {
                use base64::Engine;
                let random = sa_fox_util::random_string(32);
                base64::engine::general_purpose::STANDARD.encode(random.as_bytes())
            }
            crate::config::sa_token_config::SaTokenStyle::Jwt => {
                // JWT Token 需要 jwt 插件支持，此处降级为 UUID
                uuid::Uuid::new_v4().to_string()
            }
        }
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
