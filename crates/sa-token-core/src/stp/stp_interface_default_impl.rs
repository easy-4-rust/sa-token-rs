//! `StpInterfaceDefaultImpl` —— 1:1 对应 Java `cn.dev33.satoken.stp.StpInterfaceDefaultImpl`

use super::stp_interface::StpInterface;

/// 默认权限数据源实现（返回空列表）
pub struct StpInterfaceDefaultImpl;

impl Default for StpInterfaceDefaultImpl {
    fn default() -> Self {
        Self
    }
}

impl StpInterface for StpInterfaceDefaultImpl {
    fn get_permission_list(&self, _login_id: &str, _login_type: &str) -> Vec<String> {
        Vec::new()
    }

    fn get_role_list(&self, _login_id: &str, _login_type: &str) -> Vec<String> {
        Vec::new()
    }
}
