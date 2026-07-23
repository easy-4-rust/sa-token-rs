//! `SaResult` —— 1:1 对应 Java `cn.dev33.satoken.util.SaResult`
//!
//! 用于在 Java 端封装统一返回结构 `{ code: int, message: String, data: T }`。
//! Rust 端以 `serde::Serialize` 形式表达，可直接序列化为 JSON 返回前端。

use serde::{Deserialize, Serialize};

/// Sa-Token 通用返回结构
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SaResultData<T> {
    /// 业务码（默认 200 为成功）
    pub code: i32,
    /// 提示信息
    #[serde(default)]
    pub message: String,
    /// 数据载荷
    #[serde(default)]
    pub data: Option<T>,
}

impl<T> SaResultData<T> {
    /// 成功响应
    pub fn ok(data: T) -> Self {
        Self {
            code: 200,
            message: "ok".into(),
            data: Some(data),
        }
    }

    /// 成功响应（无数据）
    pub fn ok_empty() -> Self
    where
        T: Default,
    {
        Self {
            code: 200,
            message: "ok".into(),
            data: Some(T::default()),
        }
    }

    /// 错误响应
    pub fn error(code: i32, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            data: None,
        }
    }

    /// 自定义 code
    pub fn with_code(mut self, code: i32) -> Self {
        self.code = code;
        self
    }

    /// 自定义 message
    pub fn with_message(mut self, message: impl Into<String>) -> Self {
        self.message = message.into();
        self
    }

    /// 设置 data
    pub fn with_data(mut self, data: T) -> Self {
        self.data = Some(data);
        self
    }

    /// 是否为成功响应
    pub fn is_ok(&self) -> bool {
        self.code == 200
    }
}

/// `SaResult` 别名（Java 命名）—— 表示无 data 的返回结构
pub type SaResultEmpty = SaResultData<serde_json::Value>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serialize_ok() {
        let r: SaResultData<i32> = SaResultData::ok(42);
        assert_eq!(r.code, 200);
        assert_eq!(r.data, Some(42));
        assert!(r.is_ok());
        let json = serde_json::to_string(&r).unwrap();
        assert!(json.contains("42"));
    }

    #[test]
    fn serialize_error() {
        let r: SaResultData<()> = SaResultData::error(500, "失败");
        assert_eq!(r.code, 500);
        assert_eq!(r.message, "失败");
        assert!(!r.is_ok());
    }
}
