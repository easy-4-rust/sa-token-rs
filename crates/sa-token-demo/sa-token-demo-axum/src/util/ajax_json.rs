//! Ajax 统一返回结构（对应 Java `AjaxJson`，Jackson → serde）。

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
    /// 数据总数（分页）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data_count: Option<i64>,
}

#[allow(dead_code)]
impl AjaxJson {
    /// 成功状态码
    pub const CODE_SUCCESS: i32 = 200;
    /// 错误状态码
    pub const CODE_ERROR: i32 = 500;
    /// 警告状态码
    pub const CODE_WARNING: i32 = 501;
    /// 无权限状态码
    pub const CODE_NOT_JUR: i32 = 403;
    /// 未登录状态码
    pub const CODE_NOT_LOGIN: i32 = 401;
    /// 无效请求状态码
    pub const CODE_INVALID_REQUEST: i32 = 400;

    /// 构造响应
    pub fn new(
        code: i32,
        msg: impl Into<String>,
        data: Option<Value>,
        data_count: Option<i64>,
    ) -> Self {
        Self {
            code,
            msg: msg.into(),
            data,
            data_count,
        }
    }

    /// 返回成功
    pub fn get_success() -> Self {
        Self::new(Self::CODE_SUCCESS, "ok", None, None)
    }

    /// 返回成功（自定义消息）
    pub fn get_success_msg(msg: impl Into<String>) -> Self {
        Self::new(Self::CODE_SUCCESS, msg, None, None)
    }

    /// 返回成功数据
    pub fn get_success_data(data: impl Serialize) -> Self {
        let value = serde_json::to_value(data).unwrap_or(Value::Null);
        Self::new(Self::CODE_SUCCESS, "ok", Some(value), None)
    }

    /// 返回失败
    pub fn get_error(msg: impl Into<String>) -> Self {
        Self::new(Self::CODE_ERROR, msg, None, None)
    }

    /// 返回未登录
    pub fn get_not_login() -> Self {
        Self::new(Self::CODE_NOT_LOGIN, "未登录，请登录后再次访问", None, None)
    }

    /// 返回无权限
    pub fn get_not_jur(msg: impl Into<String>) -> Self {
        Self::new(Self::CODE_NOT_JUR, msg, None, None)
    }

    /// 自定义状态码
    pub fn get(code: i32, msg: impl Into<String>) -> Self {
        Self::new(code, msg, None, None)
    }

    /// 设置 msg（链式）
    pub fn set_msg(mut self, msg: impl Into<String>) -> Self {
        self.msg = msg.into();
        self
    }

    /// 设置 data（链式）
    pub fn set_data(mut self, data: impl Serialize) -> Self {
        self.data = Some(serde_json::to_value(data).unwrap_or(Value::Null));
        self
    }
}
