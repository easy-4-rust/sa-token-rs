//! 自定义权限数据源（对应 Java `com.pj.satoken.StpInterfaceImpl`）。

use sa_token::sa_token_core::stp::StpInterface;

/// 模拟权限/角色（与 Java Freemarker demo 一致）。
pub struct StpInterfaceImpl;

impl StpInterface for StpInterfaceImpl {
    /// 返回权限码（对应 Java `getPermissionList`）。
    fn get_permission_list(&self, _login_id: &str, _login_type: &str) -> Vec<String> {
        vec!["user-add".into(), "user-delete".into(), "user-get".into()]
    }

    /// 返回角色（对应 Java `getRoleList`）。
    fn get_role_list(&self, _login_id: &str, _login_type: &str) -> Vec<String> {
        vec!["admin".into(), "ceo".into()]
    }
}
