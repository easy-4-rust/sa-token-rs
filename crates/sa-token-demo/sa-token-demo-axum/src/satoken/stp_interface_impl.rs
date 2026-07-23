//! 自定义权限验证接口扩展（对应 Java `StpInterfaceImpl`）。

use sa_token::sa_token_core::stp::StpInterface;

/// 模拟权限/角色数据源。
pub struct StpInterfaceImpl;

impl StpInterface for StpInterfaceImpl {
    /// 返回账号拥有的权限码集合。
    fn get_permission_list(&self, _login_id: &str, _login_type: &str) -> Vec<String> {
        vec![
            "101".into(),
            "user-add".into(),
            "user-delete".into(),
            "user-update".into(),
            "user-get".into(),
            "article-get".into(),
        ]
    }

    /// 返回账号拥有的角色标识集合。
    fn get_role_list(&self, _login_id: &str, _login_type: &str) -> Vec<String> {
        vec!["admin".into(), "super-admin".into()]
    }
}
