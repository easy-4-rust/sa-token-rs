//! 注解鉴权测试（对应 Java AtController）。
//!
//! Actix 适配基于 token 解析身份，权限/角色通过 StpInterface 按 login_id 校验。

use actix_web::{HttpResponse, web};
use sa_token::prelude::AsyncStpUtil;
use sa_token_web_actix::RequireLogin;

use crate::util::AjaxJson;

/// 按 login_id 检查权限。
fn has_permission(util: &AsyncStpUtil, login_id: &str, permission: &str) -> bool {
    util.logic()
        .runtime()
        .stp_interface()
        .get_permission_list(login_id, "login")
        .iter()
        .any(|p| p == permission)
}

/// 按 login_id 检查角色。
fn has_role(util: &AsyncStpUtil, login_id: &str, role: &str) -> bool {
    util.logic()
        .runtime()
        .stp_interface()
        .get_role_list(login_id, "login")
        .iter()
        .any(|r| r == role)
}

/// 登录认证 —— `/at/checkLogin`
pub async fn check_login(_login: RequireLogin) -> HttpResponse {
    HttpResponse::Ok().json(AjaxJson::get_success())
}

/// 权限认证 —— `/at/checkPermission`
pub async fn check_permission(util: web::Data<AsyncStpUtil>, login: RequireLogin) -> HttpResponse {
    if has_permission(&util, &login.0.login_id, "user-add") {
        HttpResponse::Ok().json(AjaxJson::get_success())
    } else {
        HttpResponse::Ok().json(AjaxJson::get_not_jur("无此权限：user-add"))
    }
}

/// 权限 AND —— `/at/checkPermissionAnd`
pub async fn check_permission_and(
    util: web::Data<AsyncStpUtil>,
    login: RequireLogin,
) -> HttpResponse {
    let ok = ["user-add", "user-delete", "user-update"]
        .iter()
        .all(|p| has_permission(&util, &login.0.login_id, p));
    if ok {
        HttpResponse::Ok().json(AjaxJson::get_success())
    } else {
        HttpResponse::Ok().json(AjaxJson::get_not_jur("权限不足"))
    }
}

/// 权限 OR —— `/at/checkPermissionOr`
pub async fn check_permission_or(
    util: web::Data<AsyncStpUtil>,
    login: RequireLogin,
) -> HttpResponse {
    let ok = ["user-add", "user-delete", "user-update"]
        .iter()
        .any(|p| has_permission(&util, &login.0.login_id, p));
    if ok {
        HttpResponse::Ok().json(AjaxJson::get_success())
    } else {
        HttpResponse::Ok().json(AjaxJson::get_not_jur("无此权限"))
    }
}

/// 角色认证 —— `/at/checkRole`
pub async fn check_role(util: web::Data<AsyncStpUtil>, login: RequireLogin) -> HttpResponse {
    if has_role(&util, &login.0.login_id, "admin") {
        HttpResponse::Ok().json(AjaxJson::get_success())
    } else {
        HttpResponse::Ok().json(AjaxJson::get_not_jur("无此角色：admin"))
    }
}

/// 打开二级认证 —— `/at/openSafe`
pub async fn open_safe(util: web::Data<AsyncStpUtil>, login: RequireLogin) -> HttpResponse {
    // 无请求上下文时，直接按 token 写入 safe key
    let key = format!("satoken:login:safe:token:{}", login.0.token);
    match util.logic().runtime().dao().set(&key, "ok", 200).await {
        Ok(()) => HttpResponse::Ok().json(AjaxJson::get_success()),
        Err(e) => HttpResponse::Ok().json(AjaxJson::get_error(e.to_string())),
    }
}

/// 校验二级认证 —— `/at/checkSafe`
pub async fn check_safe(util: web::Data<AsyncStpUtil>, login: RequireLogin) -> HttpResponse {
    let key = format!("satoken:login:safe:token:{}", login.0.token);
    match util.logic().runtime().dao().get(&key).await {
        Ok(Some(_)) => HttpResponse::Ok().json(AjaxJson::get_success()),
        Ok(None) => HttpResponse::Ok().json(AjaxJson::get_not_jur("未通过二级认证")),
        Err(e) => HttpResponse::Ok().json(AjaxJson::get_error(e.to_string())),
    }
}
