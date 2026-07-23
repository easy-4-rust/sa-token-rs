//! Ajax 统一返回结构（Jackson → serde）。

use serde::Serialize;
use serde_json::Value;

/// Ajax 请求返回 JSON 封装。
#[derive(Debug, Clone, Serialize)]
pub struct AjaxJson {
    /// 状态码
    pub code: i32,
    /// 描述信息
    pub msg: String,
    /// 携带对象
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

impl AjaxJson {
    /// 成功
    pub fn get_success() -> Self {
        Self {
            code: 200,
            msg: "ok".into(),
            data: None,
        }
    }

    /// 成功（消息）
    pub fn get_success_msg(msg: impl Into<String>) -> Self {
        Self {
            code: 200,
            msg: msg.into(),
            data: None,
        }
    }

    /// 成功（数据）
    pub fn get_success_data(data: impl Serialize) -> Self {
        Self {
            code: 200,
            msg: "ok".into(),
            data: Some(serde_json::to_value(data).unwrap_or(Value::Null)),
        }
    }

    /// 失败
    pub fn get_error(msg: impl Into<String>) -> Self {
        Self {
            code: 500,
            msg: msg.into(),
            data: None,
        }
    }

    /// 未登录
    pub fn get_not_login(msg: impl Into<String>) -> Self {
        Self {
            code: 401,
            msg: msg.into(),
            data: None,
        }
    }

    /// 无权限
    pub fn get_not_jur(msg: impl Into<String>) -> Self {
        Self {
            code: 403,
            msg: msg.into(),
            data: None,
        }
    }

    /// 设置 data
    pub fn set_data(mut self, data: impl Serialize) -> Self {
        self.data = Some(serde_json::to_value(data).unwrap_or(Value::Null));
        self
    }
}
