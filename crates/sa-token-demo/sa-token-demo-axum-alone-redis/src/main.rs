//! Alone-Redis Demo：Sa-Token 独立 Redis + 业务侧另一把 Redis

mod util;

use std::env;
use std::sync::Arc;

use axum::Json;
use axum::Router;
use axum::extract::{Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use redis::AsyncCommands;
use redis::aio::ConnectionManager;
use sa_token::prelude::{AsyncSaTokenRuntime, AsyncStpUtil, SaTokenConfig};
use sa_token_core::context::sa_token_context_default_impl::SaTokenContextDefaultImpl;
use sa_token_dao_redis::SaTokenDaoRedis;
use serde::Deserialize;
use serde_json::json;

use crate::util::AjaxJson;

#[derive(Clone)]
struct AppState {
    util: AsyncStpUtil,
    biz: ConnectionManager,
}

struct AppError(String);
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        (StatusCode::OK, Json(AjaxJson::error(self.0))).into_response()
    }
}

#[derive(Debug, Deserialize)]
struct DoLoginQuery {
    #[serde(default)]
    name: String,
    #[serde(default)]
    pwd: String,
}

#[derive(Debug, Deserialize)]
struct BizQuery {
    #[serde(default)]
    key: String,
    #[serde(default)]
    value: String,
}

fn token_from_headers(headers: &HeaderMap) -> Option<String> {
    headers
        .get("satoken")
        .and_then(|v| v.to_str().ok())
        .map(str::to_string)
}

/// 登录（Sa-Token Redis DAO）。
async fn do_login(
    State(state): State<AppState>,
    Query(q): Query<DoLoginQuery>,
) -> Result<Json<AjaxJson>, AppError> {
    if q.name != "zhang" || q.pwd != "123456" {
        return Ok(Json(AjaxJson::error("登录失败")));
    }
    let token = state
        .util
        .login("10001")
        .await
        .map_err(|e| AppError(e.to_string()))?;
    Ok(Json(
        AjaxJson::ok_msg("登录成功").set_data(json!({"token": token})),
    ))
}

/// 是否登录。
async fn is_login(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<AjaxJson>, AppError> {
    let logged = match token_from_headers(&headers) {
        Some(t) => state
            .util
            .get_login_id_by_token(&t)
            .await
            .map_err(|e| AppError(e.to_string()))?
            .is_some(),
        None => false,
    };
    Ok(Json(AjaxJson::ok_data(logged)))
}

/// 注销。
async fn logout(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<AjaxJson>, AppError> {
    let token = token_from_headers(&headers).ok_or_else(|| AppError("未登录".into()))?;
    let id = state
        .util
        .get_login_id_by_token(&token)
        .await
        .map_err(|e| AppError(e.to_string()))?
        .ok_or_else(|| AppError("token 无效".into()))?;
    state
        .util
        .logout_by_login_id(&id)
        .await
        .map_err(|e| AppError(e.to_string()))?;
    Ok(Json(AjaxJson::ok()))
}

/// 业务 Redis SET。
async fn biz_set(
    State(state): State<AppState>,
    Query(q): Query<BizQuery>,
) -> Result<Json<AjaxJson>, AppError> {
    let mut conn = state.biz.clone();
    conn.set::<_, _, ()>(&q.key, &q.value)
        .await
        .map_err(|e| AppError(e.to_string()))?;
    Ok(Json(AjaxJson::ok_msg("biz set ok")))
}

/// 业务 Redis GET。
async fn biz_get(
    State(state): State<AppState>,
    Query(q): Query<BizQuery>,
) -> Result<Json<AjaxJson>, AppError> {
    let mut conn = state.biz.clone();
    let value: Option<String> = conn
        .get(&q.key)
        .await
        .map_err(|e| AppError(e.to_string()))?;
    Ok(Json(AjaxJson::ok_data(
        json!({"key": q.key, "value": value}),
    )))
}

#[tokio::main]
async fn main() {
    let satoken_url = env::var("SA_TOKEN_REDIS_URL")
        .or_else(|_| env::var("REDIS_URL"))
        .unwrap_or_else(|_| "redis://127.0.0.1:6379/0".into());
    let biz_url = env::var("BIZ_REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379/1".into());
    println!("Sa-Token Redis: {satoken_url}");
    println!("Business Redis: {biz_url}");

    let satoken_client =
        redis::Client::open(satoken_url.as_str()).expect("invalid SA_TOKEN_REDIS_URL");
    let dao = SaTokenDaoRedis::connect(satoken_client)
        .await
        .expect("satoken redis connect");
    let runtime = AsyncSaTokenRuntime::new(
        Arc::new(SaTokenConfig {
            token_name: "satoken".into(),
            timeout: 2_592_000,
            is_log: true,
            ..Default::default()
        }),
        Arc::new(dao),
        Arc::new(SaTokenContextDefaultImpl),
    );
    let util = AsyncStpUtil::new("login", Arc::new(runtime));

    let biz_client = redis::Client::open(biz_url.as_str()).expect("invalid BIZ_REDIS_URL");
    let biz = ConnectionManager::new(biz_client)
        .await
        .expect("biz redis connect");

    let state = AppState { util, biz };
    let app = Router::new()
        .route("/acc/doLogin", get(do_login).post(do_login))
        .route("/acc/isLogin", get(is_login))
        .route("/acc/logout", get(logout).post(logout))
        .route("/biz/set", get(biz_set))
        .route("/biz/get", get(biz_get))
        .with_state(state);

    let addr = "0.0.0.0:8113";
    println!("🚀 Alone-Redis Axum Demo  http://{addr}");
    let listener = tokio::net::TcpListener::bind(addr).await.expect("bind");
    axum::serve(listener, app).await.expect("serve");
}
