//! `SaTokenEventCenter` —— 1:1 对应 Java `cn.dev33.satoken.listener.SaTokenEventCenter`
//!
//! 事件中心，负责发布登录/注销/踢下线等业务事件。

use std::sync::Arc;

use super::sa_token_listener::SaTokenListener;
use crate::stp::parameter::sa_login_parameter::SaLoginParameter as LoginParam;

/// 事件中心（单例）
pub struct SaTokenEventCenter;

impl SaTokenEventCenter {
    /// 触发登录事件
    pub fn do_login(
        login_type: &str,
        login_id: &str,
        token_value: &str,
        login_parameter: &LoginParam,
        listeners: &[Arc<dyn SaTokenListener>],
    ) {
        for l in listeners.iter() {
            l.do_login(login_type, login_id, token_value, login_parameter);
        }
    }

    /// 触发注销事件
    pub fn do_logout(
        login_type: &str,
        login_id: &str,
        token_value: &str,
        listeners: &[Arc<dyn SaTokenListener>],
    ) {
        for l in listeners.iter() {
            l.do_logout(login_type, login_id, token_value);
        }
    }

    /// 触发踢下线事件
    pub fn do_kickout(
        login_type: &str,
        login_id: &str,
        token_value: &str,
        listeners: &[Arc<dyn SaTokenListener>],
    ) {
        for l in listeners.iter() {
            l.do_kickout(login_type, login_id, token_value);
        }
    }

    /// 触发顶替下线事件
    pub fn do_replaced(
        login_type: &str,
        login_id: &str,
        token_value: &str,
        listeners: &[Arc<dyn SaTokenListener>],
    ) {
        for l in listeners.iter() {
            l.do_replaced(login_type, login_id, token_value);
        }
    }

    /// 触发封禁事件
    pub fn do_disable(
        login_type: &str,
        login_id: &str,
        service: &str,
        level: i32,
        disable_time: i64,
        listeners: &[Arc<dyn SaTokenListener>],
    ) {
        for l in listeners.iter() {
            l.do_disable(login_type, login_id, service, level, disable_time);
        }
    }

    /// 触发解封事件
    pub fn do_untie_disable(
        login_type: &str,
        login_id: &str,
        service: &str,
        listeners: &[Arc<dyn SaTokenListener>],
    ) {
        for l in listeners.iter() {
            l.do_untie_disable(login_type, login_id, service);
        }
    }

    /// 触发二级认证开启事件
    pub fn do_open_safe(
        login_type: &str,
        token_value: &str,
        service: &str,
        safe_time: i64,
        listeners: &[Arc<dyn SaTokenListener>],
    ) {
        for l in listeners.iter() {
            l.do_open_safe(login_type, token_value, service, safe_time);
        }
    }

    /// 触发二级认证关闭事件
    pub fn do_close_safe(
        login_type: &str,
        token_value: &str,
        service: &str,
        listeners: &[Arc<dyn SaTokenListener>],
    ) {
        for l in listeners.iter() {
            l.do_close_safe(login_type, token_value, service);
        }
    }

    /// 触发创建会话事件
    pub fn do_create_session(session_id: &str, listeners: &[Arc<dyn SaTokenListener>]) {
        for l in listeners.iter() {
            l.do_create_session(session_id);
        }
    }

    /// 触发注销会话事件
    pub fn do_logout_session(session_id: &str, listeners: &[Arc<dyn SaTokenListener>]) {
        for l in listeners.iter() {
            l.do_logout_session(session_id);
        }
    }

    /// 触发续签事件
    pub fn do_renew_timeout(
        login_type: &str,
        login_id: &str,
        token_value: &str,
        timeout: i64,
        listeners: &[Arc<dyn SaTokenListener>],
    ) {
        for l in listeners.iter() {
            l.do_renew_timeout(login_type, login_id, token_value, timeout);
        }
    }
}
