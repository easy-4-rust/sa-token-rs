//! 注解模块（对应 Java `cn.dev33.satoken.annotation`）。
//!
//! 注解宏在独立的 `sa-token-derive` crate 中实现。
//! 本模块仅保留少量运行时类型（如 SaMode 枚举）。

/// 鉴权模式：AND / OR
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SaMode {
    /// 必须满足全部条件
    And,
    /// 满足任一条件即可
    Or,
}

impl Default for SaMode {
    fn default() -> Self {
        Self::And
    }
}
