//! 权限数据源接口（对应 Java `cn.dev33.satoken.stp.StpInterface`）。

/// 权限数据源接口
///
/// 对应 Java `StpInterface`，提供角色和权限数据。
/// 使用前必须实现此接口并注册到 `SaManager`。
pub trait StpInterface: Send + Sync + 'static {
    /// 返回指定账号拥有的权限码集合
    fn get_permission_list(&self, login_id: &str, login_type: &str) -> Vec<String>;

    /// 返回指定账号拥有的角色标识集合
    fn get_role_list(&self, login_id: &str, login_type: &str) -> Vec<String>;
}

/// 默认权限数据源实现（返回空列表）
pub struct StpInterfaceDefaultImpl;

impl StpInterface for StpInterfaceDefaultImpl {
    fn get_permission_list(&self, _login_id: &str, _login_type: &str) -> Vec<String> {
        Vec::new()
    }

    fn get_role_list(&self, _login_id: &str, _login_type: &str) -> Vec<String> {
        Vec::new()
    }
}
