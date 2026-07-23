//! AjaxJson（serde）。

use serde::Serialize;
use serde_json::Value;

/// 统一返回。
#[derive(Debug, Serialize)]
pub struct AjaxJson {
    /// 状态码
    pub code: i32,
    /// 消息
    pub msg: String,
    /// 数据
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

impl AjaxJson {
    /// 成功
    pub fn ok() -> Self {
        Self {
            code: 200,
            msg: "ok".into(),
            data: None,
        }
    }

    /// 成功消息
    pub fn ok_msg(msg: impl Into<String>) -> Self {
        Self {
            code: 200,
            msg: msg.into(),
            data: None,
        }
    }

    /// 成功数据
    pub fn ok_data(data: impl Serialize) -> Self {
        Self {
            code: 200,
            msg: "ok".into(),
            data: Some(serde_json::to_value(data).unwrap_or(Value::Null)),
        }
    }

    /// 失败
    pub fn error(msg: impl Into<String>) -> Self {
        Self {
            code: 500,
            msg: msg.into(),
            data: None,
        }
    }

    /// 链式设置 data
    pub fn set_data(mut self, data: impl Serialize) -> Self {
        self.data = Some(serde_json::to_value(data).unwrap_or(Value::Null));
        self
    }
}
