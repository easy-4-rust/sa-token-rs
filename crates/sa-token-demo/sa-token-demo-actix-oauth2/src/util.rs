//! AjaxJson 统一返回结构。

use serde::Serialize;
use serde_json::Value;

/// Ajax 请求返回 JSON 封装。
#[derive(Debug, Serialize)]
pub struct AjaxJson {
    pub code: i32,
    pub msg: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

impl AjaxJson {
    /// 成功（无 data）。
    pub fn ok() -> Self {
        Self {
            code: 200,
            msg: "ok".into(),
            data: None,
        }
    }

    /// 成功并携带 data。
    pub fn ok_data(data: impl Serialize) -> Self {
        Self {
            code: 200,
            msg: "ok".into(),
            data: Some(serde_json::to_value(data).unwrap_or(Value::Null)),
        }
    }

    /// 失败。
    pub fn error(msg: impl Into<String>) -> Self {
        Self {
            code: 500,
            msg: msg.into(),
            data: None,
        }
    }

    /// 向 data 对象中写入键值（链式）。
    pub fn set(mut self, key: &str, value: impl Serialize) -> Self {
        let mut map = match self.data.take() {
            Some(Value::Object(m)) => m,
            _ => serde_json::Map::new(),
        };
        map.insert(
            key.to_string(),
            serde_json::to_value(value).unwrap_or(Value::Null),
        );
        self.data = Some(Value::Object(map));
        self
    }
}
