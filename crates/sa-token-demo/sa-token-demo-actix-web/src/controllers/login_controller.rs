//! 登录控制器（对应 Java LoginController）。

use actix_web::{HttpResponse, web};
use sa_token::prelude::AsyncStpUtil;
use serde::Deserialize;

use crate::current::exception_to_response;
use crate::util::AjaxJson;

/// 登录查询参数。
#[derive(Debug, Deserialize)]
pub struct DoLoginQuery {
    /// 用户名
    #[serde(default)]
    pub name: String,
    /// 密码
    #[serde(default)]
    pub pwd: String,
}

/// 测试登录 —— `/acc/doLogin?name=zhang&pwd=123456`
pub async fn do_login(
    util: web::Data<AsyncStpUtil>,
    query: web::Query<DoLoginQuery>,
) -> HttpResponse {
    if query.name == "zhang" && query.pwd == "123456" {
        match util.login("10001").await {
            Ok(token) => HttpResponse::Ok().json(
                AjaxJson::get_success_msg("登录成功")
                    .set_data(serde_json::json!({ "token": token })),
            ),
            Err(e) => exception_to_response(e),
        }
    } else {
        HttpResponse::Ok().json(AjaxJson::get_error("登录失败"))
    }
}

/// 是否登录 —— `/acc/isLogin`（需 satoken 头或 Cookie）
pub async fn is_login(util: web::Data<AsyncStpUtil>, req: actix_web::HttpRequest) -> HttpResponse {
    let token = extract_token(&req);
    let logged_in = match token {
        Some(t) => util
            .get_login_id_by_token(&t)
            .await
            .ok()
            .flatten()
            .is_some(),
        None => false,
    };
    HttpResponse::Ok().json(AjaxJson::get_success_msg(format!("是否登录：{logged_in}")))
}

/// Token 信息 —— `/acc/tokenInfo`
pub async fn token_info(
    util: web::Data<AsyncStpUtil>,
    identity: sa_token_web_actix::RequireLogin,
) -> HttpResponse {
    // RequireLogin 已校验 token；此处用 util 取会话信息需上下文，改用 login_id
    HttpResponse::Ok().json(AjaxJson::get_success_data(serde_json::json!({
        "login_id": identity.0.login_id,
        "token": identity.0.token,
        "token_name": util.logic().runtime().config().token_name,
    })))
}

/// 注销 —— `/acc/logout`
pub async fn logout(
    util: web::Data<AsyncStpUtil>,
    identity: sa_token_web_actix::RequireLogin,
) -> HttpResponse {
    match util.logout_by_login_id(&identity.0.login_id).await {
        Ok(()) => HttpResponse::Ok().json(AjaxJson::get_success()),
        Err(e) => exception_to_response(e),
    }
}

/// 从请求提取 token（header / cookie / Bearer）。
fn extract_token(req: &actix_web::HttpRequest) -> Option<String> {
    req.headers()
        .get("satoken")
        .and_then(|v| v.to_str().ok())
        .map(str::to_string)
        .or_else(|| req.cookie("satoken").map(|c| c.value().to_string()))
        .or_else(|| {
            req.headers()
                .get(actix_web::http::header::AUTHORIZATION)
                .and_then(|v| v.to_str().ok())
                .and_then(|v| v.strip_prefix("Bearer "))
                .map(str::to_string)
        })
}
