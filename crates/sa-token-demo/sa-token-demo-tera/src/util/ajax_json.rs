//! Ajax JSON（对应 Java demo 中的 `SaResult` / Ajax 风格，serde 序列化）。

use serde::Serialize;
use serde_json::Value;

/// 统一 JSON 响应。
#[derive(Debug, Clone, Serialize)]
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
    /// 成功。
    pub fn get_success() -> Self {
        Self {
            code: 200,
            msg: "ok".into(),
            data: None,
        }
    }

    /// 失败。
    pub fn get_error(msg: impl Into<String>) -> Self {
        Self {
            code: 500,
            msg: msg.into(),
            data: None,
        }
    }
}
