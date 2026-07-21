//! 登出参数（对应 Java `cn.dev33.satoken.stp.parameter.SaLogoutParameter`）。
use serde::{Deserialize, Serialize};

use super::enums::{sa_logout_mode::SaLogoutMode, sa_logout_range::SaLogoutRange};

/// 登出参数
///
/// 对应 Java `SaLogoutParameter`，封装登出时的所有可选参数。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaLogoutParameter {
    /// 设备类型（null 表示不限制）
    pub device_type: Option<String>,
    /// 设备 ID（null 表示不限制）
    pub device_id: Option<String>,
    /// 注销模式
    pub mode: SaLogoutMode,
    /// 注销范围
    pub range: SaLogoutRange,
    /// 是否保留冻结操作
    pub is_keep_freeze_ops: Option<bool>,
    /// 是否保留 Token-Session
    pub is_keep_token_session: Option<bool>,
}

impl Default for SaLogoutParameter {
    fn default() -> Self {
        Self {
            device_type: None,
            device_id: None,
            mode: SaLogoutMode::Logout,
            range: SaLogoutRange::Token,
            is_keep_freeze_ops: None,
            is_keep_token_session: None,
        }
    }
}

impl SaLogoutParameter {
    /// 创建登出参数
    pub fn create() -> Self {
        Self::default()
    }

    /// 设置设备类型
    pub fn set_device_type(mut self, device_type: impl Into<String>) -> Self {
        self.device_type = Some(device_type.into());
        self
    }

    /// 设置设备 ID
    pub fn set_device_id(mut self, device_id: impl Into<String>) -> Self {
        self.device_id = Some(device_id.into());
        self
    }

    /// 设置注销模式
    pub fn set_mode(mut self, mode: SaLogoutMode) -> Self {
        self.mode = mode;
        self
    }

    /// 设置注销范围
    pub fn set_range(mut self, range: SaLogoutRange) -> Self {
        self.range = range;
        self
    }

    /// 设置是否保留冻结操作
    pub fn set_is_keep_freeze_ops(mut self, is_keep_freeze_ops: bool) -> Self {
        self.is_keep_freeze_ops = Some(is_keep_freeze_ops);
        self
    }

    /// 设置是否保留 Token-Session
    pub fn set_is_keep_token_session(mut self, is_keep_token_session: bool) -> Self {
        self.is_keep_token_session = Some(is_keep_token_session);
        self
    }
}
