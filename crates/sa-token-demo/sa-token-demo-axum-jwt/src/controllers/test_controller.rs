//! JWT 测试接口（对应 Java `TestJwtController`）。

use std::collections::HashMap;
use std::sync::Arc;

use axum::Json;
use axum::extract::{Query, State};
use axum::response::{IntoResponse, Response};
use sa_token::prelude::*;
use sa_token_jwt::StpLogicJwtForSimple;
use serde::Deserialize;
use serde_json::json;

use crate::util::AjaxJson;

/// App 状态：JWT Simple 逻辑。
pub type JwtState = Arc<StpLogicJwtForSimple>;

/// 登录参数。
#[derive(Debug, Deserialize)]
pub struct LoginQuery {
    /// 账号 id
    #[serde(default = "default_id")]
    pub id: String,
}

fn default_id() -> String {
    "10001".into()
}

/// 业务错误。
pub struct AppError(SaTokenException);

impl From<SaTokenException> for AppError {
    fn from(value: SaTokenException) -> Self {
        Self(value)
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        Json(AjaxJson::error(self.0.to_string())).into_response()
    }
}

/// 登录并签发 JWT extra —— `/test/login`
pub async fn login(
    State(jwt): State<JwtState>,
    Query(q): Query<LoginQuery>,
) -> Result<Json<AjaxJson>, AppError> {
    let mut extra = HashMap::new();
    extra.insert("name".into(), json!("张三"));

    // 会话登录（DAO）
    StpUtil::login_with_param(
        &q.id,
        &SaLoginParameter::default().set_extra_data(json!({ "name": "张三" })),
    )?;
    let session_token = StpUtil::get_token_value().unwrap_or_default();

    // JWT Simple：额外签发携带 claims 的 JWT
    let jwt_token = jwt
        .create_token_value(json!(q.id), extra)
        .map_err(|e| SaTokenException::other(e.to_string()))?;

    Ok(Json(AjaxJson::ok_data(json!({
        "session_token": session_token,
        "jwt_token": jwt_token,
    }))))
}

/// Token 信息 —— `/test/tokenInfo`
pub async fn token_info() -> Result<Json<AjaxJson>, AppError> {
    let info = StpUtil::get_token_info()?;
    Ok(Json(AjaxJson::ok_data(info)))
}

/// Session —— `/test/session`
pub async fn session() -> Result<Json<AjaxJson>, AppError> {
    let mut session = StpUtil::get_session()?;
    session.set("name", json!(now_secs()));
    SaManager::sa_token_dao().update_session(&session)?;
    Ok(Json(AjaxJson::ok_data(session)))
}

/// 需登录探测 —— `/test/test`
pub async fn test(State(jwt): State<JwtState>) -> Result<Json<AjaxJson>, AppError> {
    StpUtil::check_login()?;
    let token = StpUtil::get_token_value().unwrap_or_default();
    let name = jwt.get_extra(&token, "name").ok().flatten().or_else(|| {
        // session token 不是 JWT 时，从 extra 读不到属正常；演示用
        None
    });
    Ok(Json(AjaxJson::ok_data(json!({
        "login_id": StpUtil::get_login_id()?,
        "jwt_name_claim": name,
    }))))
}

fn now_secs() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}
