//! 全局异常处理（对应 Java `GlobalException` / `@ControllerAdvice`）。

use axum::Json;
use axum::response::{IntoResponse, Response};
use sa_token::prelude::SaTokenException;

use crate::util::AjaxJson;

/// Demo 业务错误，映射 Sa-Token 异常到 AjaxJson。
#[derive(Debug)]
pub struct AppError(pub SaTokenException);

impl From<SaTokenException> for AppError {
    fn from(value: SaTokenException) -> Self {
        Self(value)
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let aj = match &self.0 {
            SaTokenException::NotLogin { message, .. } => {
                AjaxJson::get_not_login().set_msg(message.clone())
            }
            SaTokenException::NotRole { role, .. } => {
                AjaxJson::get_not_jur(format!("无此角色：{role}"))
            }
            SaTokenException::NotPermission { permission, .. } => {
                AjaxJson::get_not_jur(format!("无此权限：{permission}"))
            }
            SaTokenException::DisableService { disable_time, .. } => {
                AjaxJson::get_not_jur(format!("账号被封禁：{disable_time}秒后解封"))
            }
            other => AjaxJson::get_error(other.to_string()),
        };
        Json(aj).into_response()
    }
}

/// Handler 统一返回类型。
pub type AppResult<T> = Result<T, AppError>;
