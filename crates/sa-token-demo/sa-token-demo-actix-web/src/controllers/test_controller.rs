//! 综合能力测试（对应 Java TestController）。

use actix_web::{HttpResponse, web};
use sa_token::prelude::AsyncStpUtil;
use sa_token_web_actix::{OptionalLogin, RequireLogin};
use serde::Deserialize;
use serde_json::json;

use crate::current::exception_to_response;
use crate::util::AjaxJson;

/// 登录参数。
#[derive(Debug, Deserialize)]
pub struct LoginQuery {
    /// 账号 id
    #[serde(default = "default_login_id")]
    pub id: String,
}

fn default_login_id() -> String {
    "10001".into()
}

/// 设备登录参数。
#[derive(Debug, Deserialize)]
pub struct LoginDeviceQuery {
    /// 账号 id
    #[serde(default = "default_login_id")]
    pub id: String,
    /// 设备类型
    #[serde(default = "default_device")]
    pub device: String,
}

fn default_device() -> String {
    "PC".into()
}

/// 测试登录 —— `/test/login`
pub async fn login(util: web::Data<AsyncStpUtil>, query: web::Query<LoginQuery>) -> HttpResponse {
    match util.login(&query.id).await {
        Ok(token) => HttpResponse::Ok().json(AjaxJson::get_success().set_data(json!({
            "token": token,
            "login_id": query.id,
        }))),
        Err(e) => exception_to_response(e),
    }
}

/// 退出登录 —— `/test/logout`
pub async fn logout(util: web::Data<AsyncStpUtil>, login: RequireLogin) -> HttpResponse {
    match util.logout_by_login_id(&login.0.login_id).await {
        Ok(()) => HttpResponse::Ok().json(AjaxJson::get_success()),
        Err(e) => exception_to_response(e),
    }
}

/// 测试角色 —— `/test/testRole`
pub async fn test_role(util: web::Data<AsyncStpUtil>, login: RequireLogin) -> HttpResponse {
    let roles = util
        .logic()
        .runtime()
        .stp_interface()
        .get_role_list(&login.0.login_id, "login");
    if roles.iter().any(|r| r == "admin") {
        HttpResponse::Ok().json(AjaxJson::get_success_msg("角色测试通过").set_data(roles))
    } else {
        HttpResponse::Ok().json(AjaxJson::get_not_jur("无此角色：admin"))
    }
}

/// 测试权限 —— `/test/testJur`
pub async fn test_jur(util: web::Data<AsyncStpUtil>, login: RequireLogin) -> HttpResponse {
    let perms = util
        .logic()
        .runtime()
        .stp_interface()
        .get_permission_list(&login.0.login_id, "login");
    if perms.iter().any(|p| p == "user-add") {
        HttpResponse::Ok().json(AjaxJson::get_success_msg("权限测试通过").set_data(perms))
    } else {
        HttpResponse::Ok().json(AjaxJson::get_not_jur("无此权限：user-add"))
    }
}

/// Account-Session —— `/test/session`
pub async fn session(util: web::Data<AsyncStpUtil>, login: RequireLogin) -> HttpResponse {
    match util.get_session_by_login_id(&login.0.login_id).await {
        Ok(mut session) => {
            session.set("name", json!("actix-demo"));
            if let Err(e) = util.logic().runtime().dao().update_session(&session).await {
                return exception_to_response(e);
            }
            HttpResponse::Ok().json(AjaxJson::get_success_data(session))
        }
        Err(e) => exception_to_response(e),
    }
}

/// Token 信息 —— `/test/tokenInfo`
pub async fn token_info(login: RequireLogin) -> HttpResponse {
    HttpResponse::Ok().json(AjaxJson::get_success_data(json!({
        "login_id": login.0.login_id,
        "token_value": login.0.token,
    })))
}

/// 踢人下线 —— `/test/kickOut`
pub async fn kick_out(util: web::Data<AsyncStpUtil>) -> HttpResponse {
    if let Err(e) = util.login("10001").await {
        return exception_to_response(e);
    }
    match util.kickout_by_login_id("10001").await {
        Ok(()) => HttpResponse::Ok().json(AjaxJson::get_success()),
        Err(e) => exception_to_response(e),
    }
}

/// 按设备登录 —— `/test/login2`
pub async fn login2(
    util: web::Data<AsyncStpUtil>,
    query: web::Query<LoginDeviceQuery>,
) -> HttpResponse {
    use sa_token::prelude::SaLoginParameter;
    let param = SaLoginParameter::default().set_device_type(&query.device);
    match util.login_with_param(&query.id, &param).await {
        Ok(token) => HttpResponse::Ok().json(AjaxJson::get_success().set_data(json!({
            "token": token,
            "device": query.device,
        }))),
        Err(e) => exception_to_response(e),
    }
}

/// 可选登录探测 —— `/test/test`
pub async fn test(optional: OptionalLogin) -> HttpResponse {
    HttpResponse::Ok().json(AjaxJson::get_success_data(json!({
        "logged_in": optional.0.is_some(),
        "login_id": optional.0.map(|i| i.login_id),
    })))
}

/// 简单测试 —— `/test/test2`
pub async fn test2() -> HttpResponse {
    HttpResponse::Ok().json(AjaxJson::get_success())
}
