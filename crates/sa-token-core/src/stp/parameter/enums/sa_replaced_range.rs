//! 顶替范围枚举（对应 Java `SaReplacedRange`）。
use serde::{Deserialize, Serialize};

/// 顶替范围
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SaReplacedRange {
    /// 仅顶替当前设备类型
    CurrDeviceType,
    /// 顶替所有设备类型
    AllDeviceType,
}

impl Default for SaReplacedRange {
    fn default() -> Self {
        Self::CurrDeviceType
    }
}
