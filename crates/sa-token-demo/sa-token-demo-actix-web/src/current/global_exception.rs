//! 全局异常映射。

use actix_web::HttpResponse;
use sa_token::prelude::SaTokenException;

use crate::util::AjaxJson;

/// 将 SaTokenException 转为 HTTP JSON 响应。
pub fn exception_to_response(err: SaTokenException) -> HttpResponse {
    let body = match &err {
        SaTokenException::NotLogin { message, .. } => AjaxJson::get_not_login(message.clone()),
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
    HttpResponse::Ok().json(body)
}
