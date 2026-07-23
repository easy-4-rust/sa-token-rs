//! 综合能力测试（对应 Java `TestController`）。

use axum::Json;
use axum::extract::Query;
use sa_token::prelude::*;
use sa_token::sa_token_core::session::sa_session_custom_util::SaSessionCustomUtil;
use serde::Deserialize;
use serde_json::json;

use crate::current::AppResult;
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
pub async fn login(Query(q): Query<LoginQuery>) -> AppResult<Json<AjaxJson>> {
    println!("======================= 进入方法，测试登录接口 =========================");
    println!("当前会话的token：{:?}", StpUtil::get_token_value());
    println!("当前是否登录：{:?}", StpUtil::is_login());
    println!("当前登录账号：{:?}", StpUtil::get_login_id_default_null());

    StpUtil::login(&q.id)?;
    let token = StpUtil::get_token_value().unwrap_or_default();
    println!("登录成功，token={token}");
    println!("当前是否登录：{:?}", StpUtil::is_login());
    println!("当前登录账号：{:?}", StpUtil::get_login_id());
    println!("当前登录设备：{:?}", StpUtil::get_login_device_type());

    Ok(Json(
        AjaxJson::get_success().set_data(json!({ "token": token })),
    ))
}

/// 退出登录 —— `/test/logout`
pub async fn logout() -> AppResult<Json<AjaxJson>> {
    StpUtil::logout()?;
    Ok(Json(AjaxJson::get_success()))
}

/// 测试角色 —— `/test/testRole`
pub async fn test_role() -> AppResult<Json<AjaxJson>> {
    println!("是否具有角色标识 user {:?}", StpUtil::has_role("user"));
    println!("是否具有角色标识 admin {:?}", StpUtil::has_role("admin"));
    StpUtil::check_role("admin")?;
    StpUtil::check_role_or(&["admin", "user"])?;
    StpUtil::check_role_and(&["admin", "super-admin"])?;
    Ok(Json(AjaxJson::get_success_msg("角色测试通过")))
}

/// 测试权限 —— `/test/testJur`
pub async fn test_jur() -> AppResult<Json<AjaxJson>> {
    println!("是否具有权限101 {:?}", StpUtil::has_permission("101"));
    println!(
        "是否具有权限user-add {:?}",
        StpUtil::has_permission("user-add")
    );
    StpUtil::check_permission("user-add")?;
    StpUtil::check_permission_or(&["101", "102"])?;
    StpUtil::check_permission_and(&["101", "user-add"])?;
    Ok(Json(AjaxJson::get_success_msg("权限测试通过")))
}

/// Account-Session —— `/test/session`
pub async fn session() -> AppResult<Json<AjaxJson>> {
    let mut session = StpUtil::get_session()?;
    println!("当前登录账号 session id={}", session.id());
    println!("测试取值 name：{:?}", session.get("name"));
    session.set("name", json!(chrono_now()));
    SaManager::sa_token_dao().update_session(&session)?;
    println!("测试取值 name：{:?}", session.get("name"));
    Ok(Json(AjaxJson::get_success_data(&session)))
}

/// 自定义 Session —— `/test/session2`
pub async fn session2() -> AppResult<Json<AjaxJson>> {
    let session_arc = SaSessionCustomUtil::get_session_by_id("1895544896")?;
    let mut session = (*session_arc).clone();
    println!("自定义 session id={}", session.id());
    println!("测试取值 name：{:?}", session.get("name"));
    session.set("name", json!("张三"));
    SaManager::sa_token_dao().update_session(&session)?;
    println!("测试取值 name：{:?}", session.get("name"));
    Ok(Json(AjaxJson::get_success()))
}

/// Token-Session —— `/test/getTokenSession`
pub async fn get_token_session() -> AppResult<Json<AjaxJson>> {
    let mut session = StpUtil::get_token_session()?;
    println!("当前 token 专属 session: {}", session.id());
    println!("测试取值 name：{:?}", session.get("name"));
    session.set("name", json!("张三"));
    SaManager::sa_token_dao().update_session(&session)?;
    Ok(Json(AjaxJson::get_success()))
}

/// Token 信息 —— `/test/tokenInfo`
pub async fn token_info() -> AppResult<Json<AjaxJson>> {
    let info = StpUtil::get_token_info()?;
    Ok(Json(AjaxJson::get_success_data(info)))
}

/// 注解式鉴权组合 —— `/test/atCheck`
pub async fn at_check() -> AppResult<Json<AjaxJson>> {
    StpUtil::check_login()?;
    StpUtil::check_role("super-admin")?;
    StpUtil::check_permission("user-add")?;
    Ok(Json(AjaxJson::get_success()))
}

/// 权限 OR 注解 —— `/test/atJurOr`
pub async fn at_jur_or() -> AppResult<Json<AjaxJson>> {
    StpUtil::check_permission_or(&["user-add", "user-all", "user-delete"])?;
    Ok(Json(AjaxJson::get_success_data("用户信息")))
}

/// 活动时间续签 —— `/test/rene`
pub async fn rene() -> AppResult<Json<AjaxJson>> {
    StpUtil::check_active_timeout()?;
    StpUtil::update_last_active_to_now()?;
    Ok(Json(AjaxJson::get_success_msg("续签成功")))
}

/// 踢人下线 —— `/test/kickOut`
pub async fn kick_out() -> AppResult<Json<AjaxJson>> {
    StpUtil::login("10001")?;
    StpUtil::kickout("10001")?;
    // 踢下线后再取 loginId 应失败，这里仅演示调用链
    let _ = StpUtil::get_login_id();
    Ok(Json(AjaxJson::get_success()))
}

/// 按设备登录 —— `/test/login2`
pub async fn login2(Query(q): Query<LoginDeviceQuery>) -> AppResult<Json<AjaxJson>> {
    StpUtil::login_with_device(&q.id, &q.device)?;
    let token = StpUtil::get_token_value().unwrap_or_default();
    Ok(Json(
        AjaxJson::get_success().set_data(json!({ "token": token })),
    ))
}

/// 身份临时切换 —— `/test/switchTo`
pub async fn switch_to() -> AppResult<Json<AjaxJson>> {
    println!("当前会话身份：{:?}", StpUtil::get_login_id_default_null());
    println!("是否正在身份临时切换中: {:?}", StpUtil::is_switch());
    StpUtil::switch_to("10044")?;
    println!("是否正在身份临时切换中: {:?}", StpUtil::is_switch());
    println!("当前会话身份已被切换为：{:?}", StpUtil::get_login_id());
    StpUtil::end_switch()?;
    println!("是否正在身份临时切换中: {:?}", StpUtil::is_switch());
    Ok(Json(AjaxJson::get_success()))
}

/// 指定设备登录 —— `/test/loginByDevice`
pub async fn login_by_device() -> AppResult<Json<AjaxJson>> {
    StpUtil::login_with_device("10001", "PC")?;
    let token = StpUtil::get_token_value().unwrap_or_default();
    Ok(Json(AjaxJson::get_success_data(
        json!({ "msg": "登录成功", "token": token }),
    )))
}

/// 简单测试 —— `/test/test`
pub async fn test() -> AppResult<Json<AjaxJson>> {
    Ok(Json(AjaxJson::get_success()))
}

/// 简单测试2 —— `/test/test2`
pub async fn test2() -> AppResult<Json<AjaxJson>> {
    Ok(Json(AjaxJson::get_success()))
}

/// 返回当前秒级时间戳字符串（替代 Java `new Date()` 序列化）。
fn chrono_now() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}
