//! 注销模式枚举（对应 Java `SaLogoutMode`）。
use serde::{Deserialize, Serialize};

/// 注销模式
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SaLogoutMode {
    /// 正常注销
    Logout,
    /// 被踢下线
    Kickout,
    /// 被顶替下线
    Replaced,
}

impl Default for SaLogoutMode {
    fn default() -> Self {
        Self::Logout
    }
}
