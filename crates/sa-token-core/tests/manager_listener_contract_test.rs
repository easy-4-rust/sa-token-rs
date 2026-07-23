//! SaManager / Listener / EventCenter / SaHolder / StpUtil 门面契约测试。
//!
//! 对应 Java：
//! - `cn.dev33.satoken.SaManager`
//! - `cn.dev33.satoken.listener.*`
//! - `cn.dev33.satoken.context.SaHolder`
//! - `cn.dev33.satoken.stp.StpUtil`（常量与逻辑解析；登录流见 sa-token-test）

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use sa_token_core::config::sa_token_config::SaTokenConfig;
use sa_token_core::context::sa_holder::SaHolder;
use sa_token_core::listener::sa_token_event_center::SaTokenEventCenter;
use sa_token_core::listener::sa_token_listener::SaTokenListener;
use sa_token_core::listener::sa_token_listener_for_log::SaTokenListenerForLog;
use sa_token_core::listener::sa_token_listener_for_simple::SaTokenListenerForSimple;
use sa_token_core::sa_manager::SaManager;
use sa_token_core::stp::parameter::sa_login_parameter::SaLoginParameter;
use sa_token_core::stp::stp_logic::StpLogic;
use sa_token_core::stp::stp_util::StpUtil;

/// 计数监听器：验证 EventCenter 派发。
struct CountingListener {
    logins: AtomicUsize,
    logouts: AtomicUsize,
}

impl SaTokenListener for CountingListener {
    fn do_login(
        &self,
        _login_type: &str,
        _login_id: &str,
        _token_value: &str,
        _login_parameter: &SaLoginParameter,
    ) {
        self.logins.fetch_add(1, Ordering::SeqCst);
    }

    fn do_logout(&self, _login_type: &str, _login_id: &str, _token_value: &str) {
        self.logouts.fetch_add(1, Ordering::SeqCst);
    }
}

/// 验证 `SaManager::set_config` / `config`（对应 Java `setConfig` / `getConfig`）。
#[test]
fn sa_manager_config_roundtrip() {
    let cfg = SaTokenConfig {
        token_name: "custom-token".into(),
        ..SaTokenConfig::default()
    };
    SaManager::set_config(Arc::new(cfg));
    assert_eq!(SaManager::config().token_name, "custom-token");
    assert_eq!(SaManager::get_config().token_name, "custom-token");
}

/// 验证 StpLogic 注册与获取（对应 Java `SaManager.getStpLogic`）。
#[test]
fn sa_manager_stp_logic_registry() {
    SaManager::put_stp_logic(Arc::new(StpLogic::new("login")));
    assert!(SaManager::get_stp_logic("login").is_some());
    SaManager::remove_stp_logic("login");
    assert!(SaManager::get_stp_logic("login").is_none());
}

/// 验证 EventCenter 向监听器派发登录/注销（对应 Java `SaTokenEventCenter`）。
#[test]
fn event_center_dispatches_to_listeners() {
    let counter = Arc::new(CountingListener {
        logins: AtomicUsize::new(0),
        logouts: AtomicUsize::new(0),
    });
    let listeners: Vec<Arc<dyn SaTokenListener>> = vec![
        Arc::new(SaTokenListenerForSimple) as Arc<dyn SaTokenListener>,
        Arc::new(SaTokenListenerForLog) as Arc<dyn SaTokenListener>,
        counter.clone() as Arc<dyn SaTokenListener>,
    ];
    let param = SaLoginParameter::default();
    SaTokenEventCenter::do_login("login", "10001", "tok-1", &param, &listeners);
    SaTokenEventCenter::do_logout("login", "10001", "tok-1", &listeners);
    assert_eq!(counter.logins.load(Ordering::SeqCst), 1);
    assert_eq!(counter.logouts.load(Ordering::SeqCst), 1);
}

/// 验证 `SaHolder::get_context` 可拿到默认上下文（对应 Java `SaHolder.getContext`）。
#[test]
fn sa_holder_get_context() {
    let ctx = SaHolder::get_context();
    // 默认 ThreadLocal 上下文存在即可
    let _ = ctx;
}

/// 验证 `StpUtil::TYPE` 默认账号体系（对应 Java `StpUtil.TYPE`）。
#[test]
fn stp_util_default_type() {
    assert_eq!(StpUtil::TYPE, "login");
}
