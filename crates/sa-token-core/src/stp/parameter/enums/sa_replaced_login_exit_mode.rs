//! 顶替登录退出模式枚举（对应 Java `SaReplacedLoginExitMode`）。
use serde::{Deserialize, Serialize};

/// 顶替登录退出模式
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SaReplacedLoginExitMode {
    /// 旧设备下线
    OldDeviceOffline,
    /// 新设备不登录
    NewDeviceNotLogin,
}

impl Default for SaReplacedLoginExitMode {
    fn default() -> Self {
        Self::OldDeviceOffline
    }
}
