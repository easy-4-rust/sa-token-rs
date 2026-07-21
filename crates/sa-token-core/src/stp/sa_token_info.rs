//! Token 信息（对应 Java `cn.dev33.satoken.stp.SaTokenInfo`）。
use serde::{Deserialize, Serialize};

/// Token 信息
///
/// 对应 Java `SaTokenInfo`，封装 Token 的详细信息。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaTokenInfo {
    /// Token 名称
    pub token_name: String,
    /// Token 值
    pub token_value: String,
    /// 是否为首次创建（true 表示是新创建的，false 表示是复用的）
    pub is_created: bool,
    /// 登录账号 ID
    pub login_id: String,
    /// Token 有效期（秒）
    pub token_timeout: i64,
    /// Session 有效期（秒）
    pub session_timeout: i64,
    /// Token-Session 有效期（秒）
    pub token_session_timeout: i64,
    /// Token 活跃超时（秒）
    pub token_active_timeout: i64,
    /// 登录设备类型
    pub login_device_type: String,
    /// Token 创建时间（秒级时间戳）
    pub token_create_time: i64,
}

impl SaTokenInfo {
    /// 创建 Token 信息
    pub fn new(token_name: impl Into<String>, token_value: impl Into<String>) -> Self {
        Self {
            token_name: token_name.into(),
            token_value: token_value.into(),
            is_created: true,
            login_id: String::new(),
            token_timeout: -1,
            session_timeout: -1,
            token_session_timeout: -1,
            token_active_timeout: -1,
            login_device_type: String::new(),
            token_create_time: crate::util::sa_fox_util::now_timestamp(),
        }
    }
}
