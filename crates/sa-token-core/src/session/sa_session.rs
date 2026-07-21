//! 会话模型（对应 Java `cn.dev33.satoken.session.SaSession`）。
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::sa_terminal_info::SaTerminalInfo;

/// 会话模型
///
/// 对应 Java `SaSession`，存储账号会话信息，包括登录终端列表和自定义数据。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaSession {
    /// 会话 ID
    id: String,
    /// 会话类型
    #[serde(rename = "type")]
    session_type: String,
    /// 账号类型
    login_type: String,
    /// 登录账号 ID
    login_id: String,
    /// Token 值
    token: String,
    /// 创建时间（秒级时间戳）
    create_time: i64,
    /// 历史终端计数
    #[serde(skip)]
    history_terminal_count: i32,
    /// 自定义数据
    data_map: HashMap<String, serde_json::Value>,
    /// 终端列表
    terminal_list: Vec<SaTerminalInfo>,
}

impl SaSession {
    /// 用户数据 key
    pub const USER: &'static str = "USER";
    /// 角色列表 key
    pub const ROLE_LIST: &'static str = "ROLE_LIST";
    /// 权限列表 key
    pub const PERMISSION_LIST: &'static str = "PERMISSION_LIST";

    /// 创建会话
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            session_type: String::new(),
            login_type: String::new(),
            login_id: String::new(),
            token: String::new(),
            create_time: crate::util::sa_fox_util::now_timestamp(),
            history_terminal_count: 0,
            data_map: HashMap::new(),
            terminal_list: Vec::new(),
        }
    }

    /// 获取会话 ID
    pub fn id(&self) -> &str {
        &self.id
    }

    /// 设置会话 ID
    pub fn set_id(&mut self, id: impl Into<String>) -> &mut Self {
        self.id = id.into();
        self
    }

    /// 获取会话类型
    pub fn session_type(&self) -> &str {
        &self.session_type
    }

    /// 设置会话类型
    pub fn set_session_type(&mut self, session_type: impl Into<String>) -> &mut Self {
        self.session_type = session_type.into();
        self
    }

    /// 获取账号类型
    pub fn login_type(&self) -> &str {
        &self.login_type
    }

    /// 设置账号类型
    pub fn set_login_type(&mut self, login_type: impl Into<String>) -> &mut Self {
        self.login_type = login_type.into();
        self
    }

    /// 获取登录账号 ID
    pub fn login_id(&self) -> &str {
        &self.login_id
    }

    /// 设置登录账号 ID
    pub fn set_login_id(&mut self, login_id: impl Into<String>) -> &mut Self {
        self.login_id = login_id.into();
        self
    }

    /// 获取 Token 值
    pub fn token(&self) -> &str {
        &self.token
    }

    /// 设置 Token 值
    pub fn set_token(&mut self, token: impl Into<String>) -> &mut Self {
        self.token = token.into();
        self
    }

    /// 获取创建时间
    pub fn create_time(&self) -> i64 {
        self.create_time
    }

    /// 获取历史终端计数
    pub fn history_terminal_count(&self) -> i32 {
        self.history_terminal_count
    }

    /// 获取终端列表
    pub fn terminal_list(&self) -> &[SaTerminalInfo] {
        &self.terminal_list
    }

    /// 获取终端列表（可变引用）
    pub fn terminal_list_mut(&mut self) -> &mut Vec<SaTerminalInfo> {
        &mut self.terminal_list
    }

    /// 添加终端
    pub fn add_terminal(&mut self, terminal: SaTerminalInfo) {
        self.history_terminal_count += 1;
        self.terminal_list.push(terminal);
    }

    /// 移除终端
    pub fn remove_terminal(&mut self, token_value: &str) {
        self.terminal_list.retain(|t| t.token_value() != token_value);
    }

    /// 获取指定 Token 的终端
    pub fn get_terminal(&self, token_value: &str) -> Option<&SaTerminalInfo> {
        self.terminal_list.iter().find(|t| t.token_value() == token_value)
    }

    /// 获取指定设备类型的终端列表
    pub fn get_terminal_list_by_device_type(&self, device_type: &str) -> Vec<&SaTerminalInfo> {
        self.terminal_list
            .iter()
            .filter(|t| t.device_type() == device_type)
            .collect()
    }

    /// 遍历终端列表
    pub fn for_each_terminal<F>(&self, mut f: F)
    where
        F: FnMut(&SaTerminalInfo),
    {
        for terminal in &self.terminal_list {
            f(terminal);
        }
    }

    /// 获取数据
    pub fn get(&self, key: &str) -> Option<&serde_json::Value> {
        self.data_map.get(key)
    }

    /// 设置数据
    pub fn set(&mut self, key: impl Into<String>, value: serde_json::Value) -> &mut Self {
        self.data_map.insert(key.into(), value);
        self
    }

    /// 删除数据
    pub fn delete(&mut self, key: &str) -> Option<serde_json::Value> {
        self.data_map.remove(key)
    }

    /// 获取所有数据
    pub fn data_map(&self) -> &HashMap<String, serde_json::Value> {
        &self.data_map
    }

    /// 设置所有数据
    pub fn set_data_map(&mut self, data_map: HashMap<String, serde_json::Value>) -> &mut Self {
        self.data_map = data_map;
        self
    }

    /// 获取数据 Map 的 key 列表
    pub fn keys(&self) -> Vec<&String> {
        self.data_map.keys().collect()
    }

    /// 清空数据
    pub fn clear(&mut self) {
        self.data_map.clear();
    }
}
