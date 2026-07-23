//! `SaTokenListenerForLog` —— 1:1 对应 Java `cn.dev33.satoken.listener.SaTokenListenerForLog`
//!
//! 默认日志型监听器：所有事件都打到日志。

use super::sa_token_listener::SaTokenListener;
use crate::stp::parameter::sa_login_parameter::SaLoginParameter as LoginParam;

/// 默认日志监听器
pub struct SaTokenListenerForLog;

impl SaTokenListener for SaTokenListenerForLog {
    fn do_login(&self, login_type: &str, login_id: &str, token_value: &str, _param: &LoginParam) {
        tracing::info!(
            login_type = login_type,
            login_id = login_id,
            token = token_value,
            "账号登录"
        );
    }

    fn do_logout(&self, login_type: &str, login_id: &str, token_value: &str) {
        tracing::info!(
            login_type = login_type,
            login_id = login_id,
            token = token_value,
            "账号登出"
        );
    }

    fn do_kickout(&self, login_type: &str, login_id: &str, token_value: &str) {
        tracing::warn!(
            login_type = login_type,
            login_id = login_id,
            token = token_value,
            "账号被踢下线"
        );
    }

    fn do_replaced(&self, login_type: &str, login_id: &str, token_value: &str) {
        tracing::warn!(
            login_type = login_type,
            login_id = login_id,
            token = token_value,
            "账号被顶替下线"
        );
    }
}
