//! `StpInterface` —— 1:1 对应 Java `cn.dev33.satoken.stp.StpInterface`

use crate::model::wrapper_info::sa_disable_wrapper_info::SaDisableWrapperInfo;

/// 权限数据源接口
///
/// 对应 Java `StpInterface`，提供角色和权限数据。
/// 使用前必须实现此接口并注册到 `SaManager`。
pub trait StpInterface: Send + Sync + 'static {
    /// 返回指定账号拥有的权限码集合
    fn get_permission_list(&self, login_id: &str, login_type: &str) -> Vec<String>;

    /// 返回指定账号拥有的角色标识集合
    fn get_role_list(&self, login_id: &str, login_type: &str) -> Vec<String>;

    /// 返回指定账号在指定服务下是否被封禁（对应 Java `isDisabled` 默认实现）
    fn is_disabled(&self, _login_id: &str, _service: &str) -> SaDisableWrapperInfo {
        SaDisableWrapperInfo::create_not_disabled()
    }
}
