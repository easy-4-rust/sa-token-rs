//! 静态门面（对应 Java `cn.dev33.satoken.stp.StpUtil`）。

use crate::exception::SaResult;
use crate::manager::SaManager;
use crate::session::sa_session::SaSession;
use crate::session::sa_terminal_info::SaTerminalInfo;
use crate::stp::parameter::sa_login_parameter::SaLoginParameter;
use crate::stp::sa_token_info::SaTokenInfo;
use crate::stp::stp_logic::StpLogic;

/// Sa-Token 静态门面
///
/// 对应 Java `StpUtil`，所有方法都是对内置 `StpLogic` 的静态代理。
pub struct StpUtil;

impl StpUtil {
    /// 默认账号类型
    pub const TYPE: &'static str = "login";

    /// 获取内置 StpLogic 实例
    pub fn stp_logic() -> std::sync::Arc<StpLogic> {
        SaManager::get_stp_logic(Self::TYPE).expect("StpLogic not initialized")
    }

    // ==================== 登录 ====================

    /// 登录
    pub fn login(id: &str) -> SaResult<()> {
        Self::stp_logic().login(id)
    }

    /// 登录（指定设备类型）
    pub fn login_with_device(id: &str, device_type: &str) -> SaResult<()> {
        Self::stp_logic().login_with_device(id, device_type)
    }

    /// 登录（完整参数）
    pub fn login_with_param(id: &str, param: &SaLoginParameter) -> SaResult<()> {
        Self::stp_logic().login_with_param(id, param)
    }

    /// 创建登录会话
    pub fn create_login_session(id: &str, param: &SaLoginParameter) -> SaResult<String> {
        Self::stp_logic().create_login_session(id, param)
    }

    // ==================== 登出 ====================

    /// 注销当前会话
    pub fn logout() -> SaResult<()> {
        Self::stp_logic().logout()
    }

    /// 按 loginId 注销
    pub fn logout_by_login_id(login_id: &str) -> SaResult<()> {
        Self::stp_logic().logout_by_login_id(login_id)
    }

    /// 按 Token 注销
    pub fn logout_by_token_value(token_value: &str) -> SaResult<()> {
        Self::stp_logic().logout_by_token_value(token_value)
    }

    /// 踢人下线
    pub fn kickout(login_id: &str) -> SaResult<()> {
        Self::stp_logic().kickout_by_login_id(login_id)
    }

    /// 按 Token 踢人下线
    pub fn kickout_by_token_value(token_value: &str) -> SaResult<()> {
        Self::stp_logic().kickout_by_token_value(token_value)
    }

    /// 顶人下线
    pub fn replaced(login_id: &str) -> SaResult<()> {
        Self::stp_logic().replaced_by_login_id(login_id)
    }

    /// 按 Token 顶人下线
    pub fn replaced_by_token_value(token_value: &str) -> SaResult<()> {
        Self::stp_logic().replaced_by_token_value(token_value)
    }

    // ==================== 登录状态 ====================

    /// 是否已登录
    pub fn is_login() -> bool {
        Self::stp_logic().is_login()
    }

    /// 检查是否已登录
    pub fn check_login() -> SaResult<()> {
        Self::stp_logic().check_login()
    }

    /// 获取当前登录 ID
    pub fn get_login_id() -> SaResult<String> {
        Self::stp_logic().get_login_id()
    }

    /// 获取当前登录 ID（未登录返回 None）
    pub fn get_login_id_default_null() -> Option<String> {
        Self::stp_logic().get_login_id_default_null()
    }

    /// 获取当前登录 ID（转为 String）
    pub fn get_login_id_as_string() -> SaResult<String> {
        Self::stp_logic().get_login_id_as_string()
    }

    /// 获取当前登录 ID（转为 i32）
    pub fn get_login_id_as_i32() -> SaResult<i32> {
        Self::stp_logic().get_login_id_as_i32()
    }

    /// 获取当前登录 ID（转为 i64）
    pub fn get_login_id_as_i64() -> SaResult<i64> {
        Self::stp_logic().get_login_id_as_i64()
    }

    /// 根据 Token 获取 loginId
    pub fn get_login_id_by_token(token_value: &str) -> Option<String> {
        Self::stp_logic().get_login_id_by_token(token_value)
    }

    // ==================== Token ====================

    /// 获取当前 Token 值
    pub fn get_token_value() -> Option<String> {
        Self::stp_logic().get_token_value()
    }

    /// 获取 Token 名称
    pub fn get_token_name() -> String {
        Self::stp_logic().token_name()
    }

    /// 获取 Token 详情
    pub fn get_token_info() -> SaResult<SaTokenInfo> {
        Self::stp_logic().get_token_info()
    }

    /// 设置 Token 值
    pub fn set_token_value(token_value: &str) -> SaResult<()> {
        Self::stp_logic().set_token_value(token_value)
    }

    // ==================== 会话 ====================

    /// 获取当前会话
    pub fn get_session() -> SaResult<SaSession> {
        Self::stp_logic().get_session()
    }

    /// 按 loginId 获取会话
    pub fn get_session_by_login_id(login_id: &str) -> SaResult<SaSession> {
        Self::stp_logic().get_session_by_login_id(login_id)
    }

    /// 获取 Token-Session
    pub fn get_token_session() -> SaResult<SaSession> {
        Self::stp_logic().get_token_session()
    }

    /// 按 Token 获取 Token-Session
    pub fn get_token_session_by_token(token_value: &str) -> SaResult<SaSession> {
        Self::stp_logic().get_token_session_by_token(token_value)
    }

    // ==================== Token 超时 ====================

    /// 获取当前 Token 超时时间
    pub fn get_token_timeout() -> i64 {
        Self::stp_logic().get_token_timeout()
    }

    /// 续签当前 Token
    pub fn renew_timeout(timeout: i64) -> SaResult<()> {
        Self::stp_logic().renew_timeout(timeout)
    }

    /// 更新最后活跃时间为当前
    pub fn update_last_active_to_now() -> SaResult<()> {
        if let Some(token_value) = Self::get_token_value() {
            Self::stp_logic().set_last_active_to_now(&token_value)
        } else {
            Ok(())
        }
    }

    /// 检查活跃超时
    pub fn check_active_timeout() -> SaResult<()> {
        Self::stp_logic().check_active_timeout()
    }

    // ==================== 设备 ====================

    /// 获取当前登录设备类型
    pub fn get_login_device_type() -> SaResult<String> {
        Self::stp_logic().get_login_device_type()
    }

    /// 获取当前登录设备 ID
    pub fn get_login_device_id() -> SaResult<String> {
        Self::stp_logic().get_login_device_id()
    }

    // ==================== 终端查询 ====================

    /// 获取指定账号的终端列表
    pub fn get_terminal_list_by_login_id(login_id: &str) -> SaResult<Vec<SaTerminalInfo>> {
        Self::stp_logic().get_terminal_list_by_login_id(login_id)
    }

    /// 获取当前终端信息
    pub fn get_terminal_info() -> SaResult<SaTerminalInfo> {
        Self::stp_logic().get_terminal_info()
    }

    /// 获取指定账号的 Token 列表
    pub fn get_token_value_list_by_login_id(login_id: &str) -> Vec<String> {
        Self::stp_logic().get_token_value_list_by_login_id(login_id)
    }

    /// 获取指定账号的 Token 值
    pub fn get_token_value_by_login_id(login_id: &str) -> Option<String> {
        Self::stp_logic().get_token_value_by_login_id(login_id)
    }

    // ==================== 权限 / 角色 ====================

    /// 获取当前账号角色列表
    pub fn get_role_list() -> SaResult<Vec<String>> {
        Self::stp_logic().get_role_list()
    }

    /// 获取指定账号角色列表
    pub fn get_role_list_for(login_id: &str) -> SaResult<Vec<String>> {
        Self::stp_logic().get_role_list_for(login_id)
    }

    /// 当前账号是否具有指定角色
    pub fn has_role(role: &str) -> SaResult<bool> {
        Self::stp_logic().has_role(role)
    }

    /// 当前账号是否具有指定角色（AND 模式）
    pub fn has_role_and(roles: &[&str]) -> SaResult<bool> {
        Self::stp_logic().has_role_and(roles)
    }

    /// 当前账号是否具有指定角色（OR 模式）
    pub fn has_role_or(roles: &[&str]) -> SaResult<bool> {
        Self::stp_logic().has_role_or(roles)
    }

    /// 检查当前账号是否具有指定角色（不满足则抛异常）
    pub fn check_role(role: &str) -> SaResult<()> {
        Self::stp_logic().check_role(role)
    }

    /// 检查当前账号是否具有指定角色（AND 模式）
    pub fn check_role_and(roles: &[&str]) -> SaResult<()> {
        Self::stp_logic().check_role_and(roles)
    }

    /// 检查当前账号是否具有指定角色（OR 模式）
    pub fn check_role_or(roles: &[&str]) -> SaResult<()> {
        Self::stp_logic().check_role_or(roles)
    }

    /// 获取当前账号权限列表
    pub fn get_permission_list() -> SaResult<Vec<String>> {
        Self::stp_logic().get_permission_list()
    }

    /// 获取指定账号权限列表
    pub fn get_permission_list_for(login_id: &str) -> SaResult<Vec<String>> {
        Self::stp_logic().get_permission_list_for(login_id)
    }

    /// 当前账号是否具有指定权限
    pub fn has_permission(permission: &str) -> SaResult<bool> {
        Self::stp_logic().has_permission(permission)
    }

    /// 当前账号是否具有指定权限（AND 模式）
    pub fn has_permission_and(permissions: &[&str]) -> SaResult<bool> {
        Self::stp_logic().has_permission_and(permissions)
    }

    /// 当前账号是否具有指定权限（OR 模式）
    pub fn has_permission_or(permissions: &[&str]) -> SaResult<bool> {
        Self::stp_logic().has_permission_or(permissions)
    }

    /// 检查当前账号是否具有指定权限（不满足则抛异常）
    pub fn check_permission(permission: &str) -> SaResult<()> {
        Self::stp_logic().check_permission(permission)
    }

    /// 检查当前账号是否具有指定权限（AND 模式）
    pub fn check_permission_and(permissions: &[&str]) -> SaResult<()> {
        Self::stp_logic().check_permission_and(permissions)
    }

    /// 检查当前账号是否具有指定权限（OR 模式）
    pub fn check_permission_or(permissions: &[&str]) -> SaResult<()> {
        Self::stp_logic().check_permission_or(permissions)
    }

    // ==================== 禁用 ====================

    /// 封禁账号
    pub fn disable(login_id: &str, time: i64) -> SaResult<()> {
        Self::stp_logic().disable(login_id, time)
    }

    /// 解封账号
    pub fn untie_disable(login_id: &str) -> SaResult<()> {
        Self::stp_logic().untie_disable(login_id)
    }

    /// 获取封禁剩余时间
    pub fn get_disable_time(login_id: &str) -> i64 {
        Self::stp_logic().get_disable_time(login_id)
    }

    /// 指定账号是否被封禁
    pub fn is_disable(login_id: &str) -> bool {
        Self::stp_logic().is_disable(login_id)
    }

    /// 检查账号是否被封禁（被封禁则抛异常）
    pub fn check_disable(login_id: &str) -> SaResult<()> {
        Self::stp_logic().check_disable(login_id)
    }

    // ==================== 安全认证 ====================

    /// 开启二级认证
    pub fn open_safe(safe_time: i64) -> SaResult<()> {
        Self::stp_logic().open_safe(safe_time)
    }

    /// 开启二级认证（指定业务）
    pub fn open_safe_with_service(service: &str, safe_time: i64) -> SaResult<()> {
        Self::stp_logic().open_safe_with_service(service, safe_time)
    }

    /// 当前是否处于二级认证
    pub fn is_safe() -> bool {
        Self::stp_logic().is_safe()
    }

    /// 校验二级认证（未通过则抛异常）
    pub fn check_safe() -> SaResult<()> {
        Self::stp_logic().check_safe()
    }

    /// 关闭二级认证
    pub fn close_safe() -> SaResult<()> {
        Self::stp_logic().close_safe()
    }

    // ==================== 切换账号 ====================

    /// 临时切换到指定账号身份
    pub fn switch_to(login_id: &str) -> SaResult<()> {
        Self::stp_logic().switch_to(login_id)
    }

    /// 结束切换
    pub fn end_switch() -> SaResult<()> {
        Self::stp_logic().end_switch()
    }

    /// 当前是否处于切换状态
    pub fn is_switch() -> bool {
        Self::stp_logic().is_switch()
    }

    /// 获取临时切换的 loginId
    pub fn get_switch_login_id() -> Option<String> {
        Self::stp_logic().get_switch_login_id()
    }
}
