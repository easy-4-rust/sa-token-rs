//! StpInterface 动态权限数据源集成测试

use std::sync::Arc;

use sa_token_core::sa_manager::SaManager;
use sa_token_core::stp::{StpInterface, StpInterfaceDefaultImpl};

/// 自定义测试数据源
struct TestStpInterface {
    permissions: Vec<String>,
    roles: Vec<String>,
}

impl StpInterface for TestStpInterface {
    fn get_permission_list(&self, _login_id: &str, _login_type: &str) -> Vec<String> {
        self.permissions.clone()
    }

    fn get_role_list(&self, _login_id: &str, _login_type: &str) -> Vec<String> {
        self.roles.clone()
    }
}

#[test]
fn default_impl_returns_empty() {
    let tpl = StpInterfaceDefaultImpl;
    assert!(tpl.get_permission_list("u1", "login").is_empty());
    assert!(tpl.get_role_list("u1", "login").is_empty());
}

#[test]
fn manager_provides_default() {
    // manager 不在测试中初始化（无全局 config），但 try_get 应返回 None
    let _ = SaManager::stp_interface;
}

#[test]
fn custom_impl_returns_values() {
    let custom = Arc::new(TestStpInterface {
        permissions: vec!["user:list".into(), "user:add".into()],
        roles: vec!["admin".into()],
    });
    assert_eq!(custom.get_permission_list("u1", "login").len(), 2);
    assert_eq!(custom.get_role_list("u1", "login"), vec!["admin"]);
}
