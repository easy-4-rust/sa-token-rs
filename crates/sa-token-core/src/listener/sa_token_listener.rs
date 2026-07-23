//! `SaTokenListener` 监听器 trait —— 1:1 对应 Java `cn.dev33.satoken.listener.SaTokenListener`

use std::sync::{Arc, RwLock};

use crate::stp::parameter::sa_login_parameter::SaLoginParameter as LoginParam;

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
        _login_parameter: &LoginParam,
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
    fn do_open_safe(&self, _login_type: &str, _token_value: &str, _service: &str, _safe_time: i64) {
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

/// 事件监听器列表（对应 Java `SaTokenEventCenter`）
pub static LISTENERS: std::sync::OnceLock<RwLock<Vec<Arc<dyn SaTokenListener>>>> =
    std::sync::OnceLock::new();

/// 获取全局监听器列表
pub fn listeners() -> &'static RwLock<Vec<Arc<dyn SaTokenListener>>> {
    LISTENERS.get_or_init(|| RwLock::new(Vec::new()))
}

/// 注册全局监听器
pub fn register_listener(listener: Arc<dyn SaTokenListener>) {
    listeners().write().unwrap().push(listener);
}
