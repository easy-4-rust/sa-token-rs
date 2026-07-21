//! 监听器模块（对应 Java `cn.dev33.satoken.listener`）。
pub mod sa_token_listener;

use std::sync::{Arc, RwLock};

use crate::stp::parameter::sa_login_parameter::SaLoginParameter;

/// 事件监听器 trait
///
/// 对应 Java `SaTokenListener`，定义了 Sa-Token 的事件回调。
pub trait SaTokenListener: Send + Sync + 'static {
    /// 登录事件
    fn do_login(
        &self,
        _login_type: &str,
        _login_id: &str,
        _token_value: &str,
        _login_parameter: &SaLoginParameter,
    ) {
    }

    /// 登出事件
    fn do_logout(&self, _login_type: &str, _login_id: &str, _token_value: &str) {}

    /// 被踢下线事件
    fn do_kickout(&self, _login_type: &str, _login_id: &str, _token_value: &str) {}

    /// 被顶替下线事件
    fn do_replaced(&self, _login_type: &str, _login_id: &str, _token_value: &str) {}

    /// 被封禁事件
    fn do_disable(
        &self,
        _login_type: &str,
        _login_id: &str,
        _service: &str,
        _level: i32,
        _disable_time: i64,
    ) {
    }

    /// 解封事件
    fn do_untie_disable(&self, _login_type: &str, _login_id: &str, _service: &str) {}

    /// 开启二级认证事件
    fn do_open_safe(
        &self,
        _login_type: &str,
        _token_value: &str,
        _service: &str,
        _safe_time: i64,
    ) {
    }

    /// 关闭二级认证事件
    fn do_close_safe(&self, _login_type: &str, _token_value: &str, _service: &str) {}

    /// 创建 Session 事件
    fn do_create_session(&self, _session_id: &str) {}

    /// 注销 Session 事件
    fn do_logout_session(&self, _session_id: &str) {}

    /// 续签事件
    fn do_renew_timeout(
        &self,
        _login_type: &str,
        _login_id: &str,
        _token_value: &str,
        _timeout: i64,
    ) {
    }
}

/// 事件中心（对应 Java `SaTokenEventCenter`）
pub struct SaTokenEventCenter;

static LISTENERS: std::sync::OnceLock<RwLock<Vec<Arc<dyn SaTokenListener>>>> =
    std::sync::OnceLock::new();

impl SaTokenEventCenter {
    /// 获取监听器列表
    pub fn listeners() -> &'static RwLock<Vec<Arc<dyn SaTokenListener>>> {
        LISTENERS.get_or_init(|| RwLock::new(Vec::new()))
    }

    /// 注册监听器
    pub fn register_listener(listener: Arc<dyn SaTokenListener>) {
        Self::listeners().write().unwrap().push(listener);
    }

    /// 触发登录事件
    pub fn do_login(
        login_type: &str,
        login_id: &str,
        token_value: &str,
        param: &SaLoginParameter,
    ) {
        if let Ok(listeners) = Self::listeners().read() {
            for listener in listeners.iter() {
                listener.do_login(login_type, login_id, token_value, param);
            }
        }
    }

    /// 触发登出事件
    pub fn do_logout(login_type: &str, login_id: &str, token_value: &str) {
        if let Ok(listeners) = Self::listeners().read() {
            for listener in listeners.iter() {
                listener.do_logout(login_type, login_id, token_value);
            }
        }
    }

    /// 触发踢人事件
    pub fn do_kickout(login_type: &str, login_id: &str, token_value: &str) {
        if let Ok(listeners) = Self::listeners().read() {
            for listener in listeners.iter() {
                listener.do_kickout(login_type, login_id, token_value);
            }
        }
    }

    /// 触发顶替事件
    pub fn do_replaced(login_type: &str, login_id: &str, token_value: &str) {
        if let Ok(listeners) = Self::listeners().read() {
            for listener in listeners.iter() {
                listener.do_replaced(login_type, login_id, token_value);
            }
        }
    }
}
