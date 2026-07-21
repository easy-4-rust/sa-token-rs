//! 终端信息（对应 Java `cn.dev33.satoken.session.SaTerminalInfo`）。
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 终端信息
///
/// 描述一个登录终端的信息，包括设备类型、设备 ID 等。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaTerminalInfo {
    /// 登录会话索引值（该账号第几个登录设备，从 1 开始）
    pub index: i32,
    /// Token 值
    pub token_value: String,
    /// 设备类型（PC/WEB/HD/MOBILE/APP 等）
    pub device_type: String,
    /// 登录设备唯一标识
    pub device_id: String,
    /// 自定义扩展数据
    pub extra_data: HashMap<String, serde_json::Value>,
    /// 创建时间（秒级时间戳）
    pub create_time: i64,
    /// 认证哈希（用于踢下线）
    pub auth_hash: String,
}

impl SaTerminalInfo {
    /// 创建终端信息
    pub fn new(index: i32, token_value: impl Into<String>, device_type: impl Into<String>) -> Self {
        Self {
            index,
            token_value: token_value.into(),
            device_type: device_type.into(),
            device_id: String::new(),
            extra_data: HashMap::new(),
            create_time: crate::util::sa_fox_util::now_timestamp(),
            auth_hash: String::new(),
        }
    }

    /// 获取 Token 值
    pub fn token_value(&self) -> &str {
        &self.token_value
    }

    /// 获取设备类型
    pub fn device_type(&self) -> &str {
        &self.device_type
    }

    /// 获取设备 ID
    pub fn device_id(&self) -> &str {
        &self.device_id
    }

    /// 设置设备 ID
    pub fn set_device_id(&mut self, device_id: impl Into<String>) -> &mut Self {
        self.device_id = device_id.into();
        self
    }

    /// 获取扩展数据
    pub fn extra_data(&self) -> &HashMap<String, serde_json::Value> {
        &self.extra_data
    }

    /// 设置扩展数据
    pub fn set_extra(&mut self, key: impl Into<String>, value: serde_json::Value) -> &mut Self {
        self.extra_data.insert(key.into(), value);
        self
    }

    /// 获取扩展数据中的值
    pub fn get_extra(&self, key: &str) -> Option<&serde_json::Value> {
        self.extra_data.get(key)
    }

    /// 是否有扩展数据
    pub fn has_extra_data(&self) -> bool {
        !self.extra_data.is_empty()
    }

    /// 获取创建时间
    pub fn create_time(&self) -> i64 {
        self.create_time
    }

    /// 获取认证哈希
    pub fn auth_hash(&self) -> &str {
        &self.auth_hash
    }

    /// 设置认证哈希
    pub fn set_auth_hash(&mut self, auth_hash: impl Into<String>) -> &mut Self {
        self.auth_hash = auth_hash.into();
        self
    }
}
