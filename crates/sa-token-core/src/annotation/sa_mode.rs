//! `SaMode` —— 1:1 对应 Java `cn.dev33.satoken.annotation.SaMode`
//!
//! 多权限/角色校验时的 AND / OR 模式。

/// 多条件校验模式（对应 Java 枚举 `SaMode`）
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SaMode {
    /// 必须满足全部条件（对应 Java `SaMode.AND`，默认）
    #[default]
    And,
    /// 满足任一条件即可（对应 Java `SaMode.OR`）
    Or,
}
