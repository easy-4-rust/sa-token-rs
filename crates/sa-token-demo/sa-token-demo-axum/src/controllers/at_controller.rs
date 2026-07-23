//! 注解鉴权测试（对应 Java `AtController`）。

use axum::Json;
use sa_token::prelude::*;

use crate::current::AppResult;
use crate::util::AjaxJson;

/// 登录认证 —— `/at/checkLogin`
pub async fn check_login() -> AppResult<Json<AjaxJson>> {
    StpUtil::check_login()?;
    Ok(Json(AjaxJson::get_success()))
}

/// 权限认证 —— `/at/checkPermission`
pub async fn check_permission() -> AppResult<Json<AjaxJson>> {
    StpUtil::check_permission("user-add")?;
    Ok(Json(AjaxJson::get_success()))
}

/// 权限认证（AND）—— `/at/checkPermissionAnd`
pub async fn check_permission_and() -> AppResult<Json<AjaxJson>> {
    StpUtil::check_permission_and(&["user-add", "user-delete", "user-update"])?;
    Ok(Json(AjaxJson::get_success()))
}

/// 权限认证（OR）—— `/at/checkPermissionOr`
pub async fn check_permission_or() -> AppResult<Json<AjaxJson>> {
    StpUtil::check_permission_or(&["user-add", "user-delete", "user-update"])?;
    Ok(Json(AjaxJson::get_success()))
}

/// 角色认证 —— `/at/checkRole`
pub async fn check_role() -> AppResult<Json<AjaxJson>> {
    StpUtil::check_role("admin")?;
    Ok(Json(AjaxJson::get_success()))
}

/// 打开二级认证 —— `/at/openSafe`
pub async fn open_safe() -> AppResult<Json<AjaxJson>> {
    StpUtil::open_safe(200)?;
    Ok(Json(AjaxJson::get_success()))
}

/// 校验二级认证 —— `/at/checkSafe`
pub async fn check_safe() -> AppResult<Json<AjaxJson>> {
    StpUtil::check_safe()?;
    Ok(Json(AjaxJson::get_success()))
}
