//! 默认监听器（对应 Java `SaTokenListenerForLog`）。

use super::SaTokenListener;
use crate::stp::parameter::sa_login_parameter::SaLoginParameter;

/// 日志监听器
///
/// 对应 Java `SaTokenListenerForLog`，在事件发生时输出日志。
pub struct SaTokenListenerForLog;

impl SaTokenListener for SaTokenListenerForLog {
    fn do_login(
        &self,
        login_type: &str,
        login_id: &str,
        token_value: &str,
        _param: &SaLoginParameter,
    ) {
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
