//! 登录测试（对应 Java `LoginController`）。

use axum::Json;
use axum::extract::Query;
use sa_token::prelude::*;
use serde::Deserialize;

use crate::current::AppResult;
use crate::util::AjaxJson;

/// 登录请求参数。
#[derive(Debug, Deserialize)]
pub struct DoLoginQuery {
    /// 用户名
    #[serde(default)]
    pub name: String,
    /// 密码
    #[serde(default)]
    pub pwd: String,
}

/// 测试登录 —— `GET/POST /acc/doLogin?name=zhang&pwd=123456`
pub async fn do_login(Query(q): Query<DoLoginQuery>) -> AppResult<Json<AjaxJson>> {
    if q.name == "zhang" && q.pwd == "123456" {
        StpUtil::login("10001")?;
        let token = StpUtil::get_token_value().unwrap_or_default();
        return Ok(Json(
            AjaxJson::get_success_msg("登录成功").set_data(serde_json::json!({ "token": token })),
        ));
    }
    Ok(Json(AjaxJson::get_error("登录失败")))
}

/// 查询登录状态 —— `/acc/isLogin`
pub async fn is_login() -> AppResult<Json<AjaxJson>> {
    let logged_in = StpUtil::is_login()?;
    Ok(Json(AjaxJson::get_success_msg(format!(
        "是否登录：{logged_in}"
    ))))
}

/// 查询 Token 信息 —— `/acc/tokenInfo`
pub async fn token_info() -> AppResult<Json<AjaxJson>> {
    let info = StpUtil::get_token_info()?;
    Ok(Json(AjaxJson::get_success_data(info)))
}

/// 注销 —— `/acc/logout`
pub async fn logout() -> AppResult<Json<AjaxJson>> {
    StpUtil::logout()?;
    Ok(Json(AjaxJson::get_success()))
}
