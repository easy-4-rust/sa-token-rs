//! AjaxJson（serde）。

use serde::Serialize;
use serde_json::Value;

/// 统一返回。
#[derive(Debug, Serialize)]
pub struct AjaxJson {
    pub code: i32,
    pub msg: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

impl AjaxJson {
    pub fn ok() -> Self {
        Self {
            code: 200,
            msg: "ok".into(),
            data: None,
        }
    }
    pub fn ok_msg(msg: impl Into<String>) -> Self {
        Self {
            code: 200,
            msg: msg.into(),
            data: None,
        }
    }
    pub fn ok_data(data: impl Serialize) -> Self {
        Self {
            code: 200,
            msg: "ok".into(),
            data: Some(serde_json::to_value(data).unwrap_or(Value::Null)),
        }
    }
    pub fn error(msg: impl Into<String>) -> Self {
        Self {
            code: 500,
            msg: msg.into(),
            data: None,
        }
    }
    pub fn set_data(mut self, data: impl Serialize) -> Self {
        self.data = Some(serde_json::to_value(data).unwrap_or(Value::Null));
        self
    }
}
