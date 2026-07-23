//! Sa-Token-Rs Axum Redis Demo
//!
//! 对应 Java：`sa-token-demo-springboot-redis`
//! Redis DAO 为异步实现，故使用 AsyncStpUtil（与 Spring Boot 行为对齐，非同步 SaManager）。

mod util;

use std::env;
use std::sync::Arc;

use axum::Json;
use axum::Router;
use axum::extract::{Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use sa_token::prelude::{AsyncSaTokenRuntime, AsyncStpUtil, SaTokenConfig, SaTokenException};
use sa_token_core::context::sa_token_context_default_impl::SaTokenContextDefaultImpl;
use sa_token_core::stp::StpInterface;
use sa_token_dao_redis::SaTokenDaoRedis;
use serde::Deserialize;
use serde_json::json;

use crate::util::AjaxJson;

/// 权限数据源。
struct StpInterfaceImpl;

impl StpInterface for StpInterfaceImpl {
    fn get_permission_list(&self, _login_id: &str, _login_type: &str) -> Vec<String> {
        vec!["101".into(), "user-add".into(), "user-delete".into()]
    }

    fn get_role_list(&self, _login_id: &str, _login_type: &str) -> Vec<String> {
        vec!["admin".into(), "super-admin".into()]
    }
}

/// 登录参数。
#[derive(Debug, Deserialize)]
struct DoLoginQuery {
    #[serde(default)]
    name: String,
    #[serde(default)]
    pwd: String,
}

/// 登录 id 参数。
#[derive(Debug, Deserialize)]
struct LoginQuery {
    #[serde(default = "default_id")]
    id: String,
}

fn default_id() -> String {
    "10001".into()
}

/// 业务错误。
struct AppError(SaTokenException);

impl From<SaTokenException> for AppError {
    fn from(value: SaTokenException) -> Self {
        Self(value)
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        (StatusCode::OK, Json(AjaxJson::error(self.0.to_string()))).into_response()
    }
}

/// 从请求头提取 satoken。
fn token_from_headers(headers: &HeaderMap) -> Option<String> {
    headers
        .get("satoken")
        .and_then(|v| v.to_str().ok())
        .map(str::to_string)
        .or_else(|| {
            headers
                .get(axum::http::header::AUTHORIZATION)
                .and_then(|v| v.to_str().ok())
                .and_then(|v| v.strip_prefix("Bearer "))
                .map(str::to_string)
        })
}

/// 登录 —— `/acc/doLogin`
async fn do_login(
    State(util): State<AsyncStpUtil>,
    Query(q): Query<DoLoginQuery>,
) -> Result<Json<AjaxJson>, AppError> {
    if q.name == "zhang" && q.pwd == "123456" {
        let token = util.login("10001").await?;
        return Ok(Json(
            AjaxJson::ok_msg("登录成功").set_data(json!({ "token": token })),
        ));
    }
    Ok(Json(AjaxJson::error("登录失败")))
}

/// 测试登录 —— `/test/login`
async fn test_login(
    State(util): State<AsyncStpUtil>,
    Query(q): Query<LoginQuery>,
) -> Result<Json<AjaxJson>, AppError> {
    let token = util.login(&q.id).await?;
    Ok(Json(AjaxJson::ok().set_data(json!({ "token": token }))))
}

/// 是否登录 —— `/acc/isLogin`
async fn is_login(
    State(util): State<AsyncStpUtil>,
    headers: HeaderMap,
) -> Result<Json<AjaxJson>, AppError> {
    let logged_in = match token_from_headers(&headers) {
        Some(t) => util.get_login_id_by_token(&t).await?.is_some(),
        None => false,
    };
    Ok(Json(AjaxJson::ok_msg(format!("是否登录：{logged_in}"))))
}

/// 注销 —— `/acc/logout`
async fn logout(
    State(util): State<AsyncStpUtil>,
    headers: HeaderMap,
) -> Result<Json<AjaxJson>, AppError> {
    let token = token_from_headers(&headers)
        .ok_or_else(|| SaTokenException::not_login("未登录", "login"))?;
    let login_id = util
        .get_login_id_by_token(&token)
        .await?
        .ok_or_else(|| SaTokenException::not_login("token 无效", "login"))?;
    util.logout_by_login_id(&login_id).await?;
    Ok(Json(AjaxJson::ok()))
}

/// Token 信息 —— `/test/tokenInfo`
async fn token_info(
    State(util): State<AsyncStpUtil>,
    headers: HeaderMap,
) -> Result<Json<AjaxJson>, AppError> {
    let token = token_from_headers(&headers)
        .ok_or_else(|| SaTokenException::not_login("未登录", "login"))?;
    let login_id = util
        .get_login_id_by_token(&token)
        .await?
        .ok_or_else(|| SaTokenException::not_login("token 无效", "login"))?;
    Ok(Json(AjaxJson::ok_data(json!({
        "token_value": token,
        "login_id": login_id,
        "token_name": util.logic().runtime().config().token_name,
    }))))
}

/// 连接 Redis 并启动。
#[tokio::main]
async fn main() {
    let redis_url = env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".into());
    println!("Connecting Redis: {redis_url}");

    let client = redis::Client::open(redis_url.as_str()).expect("invalid REDIS_URL");
    let dao = SaTokenDaoRedis::connect(client)
        .await
        .expect("redis connect failed");

    let runtime = AsyncSaTokenRuntime::new(
        Arc::new(SaTokenConfig {
            token_name: "satoken".into(),
            timeout: 2_592_000,
            is_concurrent: true,
            is_share: false,
            is_log: true,
            ..Default::default()
        }),
        Arc::new(dao),
        Arc::new(SaTokenContextDefaultImpl),
    )
    .with_stp_interface(Arc::new(StpInterfaceImpl));

    let util = AsyncStpUtil::new("login", Arc::new(runtime));

    let app = Router::new()
        .route("/acc/doLogin", get(do_login).post(do_login))
        .route("/acc/isLogin", get(is_login))
        .route("/acc/logout", get(logout).post(logout))
        .route("/test/login", get(test_login))
        .route("/test/tokenInfo", get(token_info))
        .with_state(util);

    let addr = "0.0.0.0:8085";
    println!("🚀 Sa-Token-Rs Axum Redis Demo（对应 springboot-redis）");
    println!("   http://{addr}");
    println!("   环境变量 REDIS_URL 可覆盖连接（默认 redis://127.0.0.1:6379）");

    let listener = tokio::net::TcpListener::bind(addr).await.expect("bind");
    axum::serve(listener, app).await.expect("serve");
}
