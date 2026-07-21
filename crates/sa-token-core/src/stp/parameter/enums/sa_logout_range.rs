//! 注销范围枚举（对应 Java `SaLogoutRange`）。
use serde::{Deserialize, Serialize};

/// 注销范围
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SaLogoutRange {
    /// 仅注销当前 Token
    Token,
    /// 注销该账号所有会话
    Account,
}

impl Default for SaLogoutRange {
    fn default() -> Self {
        Self::Token
    }
}
