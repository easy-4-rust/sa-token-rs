//! `SaTokenListenerForSimple` —— 1:1 对应 Java `cn.dev33.satoken.listener.SaTokenListenerForSimple`

use super::sa_token_listener::SaTokenListener;
use crate::stp::parameter::sa_login_parameter::SaLoginParameter as LoginParam;

/// 默认的「简单实现」监听器（所有方法空实现，便于继承重写）
pub struct SaTokenListenerForSimple;

impl SaTokenListener for SaTokenListenerForSimple {
    fn do_login(
        &self,
        _login_type: &str,
        _login_id: &str,
        _token_value: &str,
        _login_parameter: &LoginParam,
    ) {
    }
    fn do_logout(&self, _login_type: &str, _login_id: &str, _token_value: &str) {}
    fn do_kickout(&self, _login_type: &str, _login_id: &str, _token_value: &str) {}
    fn do_replaced(&self, _login_type: &str, _login_id: &str, _token_value: &str) {}
    fn do_disable(
        &self,
        _login_type: &str,
        _login_id: &str,
        _service: &str,
        _level: i32,
        _disable_time: i64,
    ) {
    }
    fn do_untie_disable(&self, _login_type: &str, _login_id: &str, _service: &str) {}
    fn do_open_safe(&self, _login_type: &str, _token_value: &str, _service: &str, _safe_time: i64) {
    }
    fn do_close_safe(&self, _login_type: &str, _token_value: &str, _service: &str) {}
    fn do_create_session(&self, _session_id: &str) {}
    fn do_logout_session(&self, _session_id: &str) {}
    fn do_renew_timeout(
        &self,
        _login_type: &str,
        _login_id: &str,
        _token_value: &str,
        _timeout: i64,
    ) {
    }
}
