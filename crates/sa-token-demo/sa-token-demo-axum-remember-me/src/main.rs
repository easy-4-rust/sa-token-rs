//! Sa-Token-Rs Axum Remember-Me Demo
//!
//! 对应 Java：`sa-token-demo-remember-me`
//! Rust 无 `login(id, bool)`，请用 `SaLoginParameter::set_is_lasting_cookie`。

mod util;

use std::sync::Arc;

use axum::Json;
use axum::Router;
use axum::extract::{Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use sa_token::prelude::{
    AsyncSaTokenRuntime, AsyncStpUtil, SaLoginParameter, SaTokenConfig, SaTokenDaoMemory,
    SaTokenException,
};
use sa_token_core::context::mock::sa_request_for_mock::SaRequestForMock;
use sa_token_core::context::mock::sa_response_for_mock::SaResponseForMock;
use sa_token_core::context::mock::sa_storage_for_mock::SaStorageForMock;
use sa_token_core::context::model::sa_request::SaRequest;
use sa_token_core::context::model::sa_response::SaResponse;
use sa_token_core::context::model::sa_storage::SaStorage;
use sa_token_core::context::model::sa_token_context_model_box::SaTokenContextModelBox;
use sa_token_core::context::sa_token_context::SaTokenContext;
use serde::Deserialize;
use serde_json::json;

use crate::util::AjaxJson;

/// 演示用请求上下文：始终有效。
struct DemoContext {
    request: Arc<dyn SaRequest>,
    response: Arc<dyn SaResponse>,
    storage: Arc<dyn SaStorage>,
}

impl DemoContext {
    /// 创建空 Mock 上下文。
    fn new() -> Self {
        Self {
            request: Arc::new(SaRequestForMock::default()),
            response: Arc::new(SaResponseForMock::new()),
            storage: Arc::new(SaStorageForMock::new()),
        }
    }
}

impl SaTokenContext for DemoContext {
    fn set_context(
        &self,
        _request: Arc<dyn SaRequest>,
        _response: Arc<dyn SaResponse>,
        _storage: Arc<dyn SaStorage>,
    ) {
    }

    fn clear_context(&self) {
        self.storage.delete("satoken");
    }

    fn is_valid(&self) -> bool {
        true
    }

    fn model_box(&self) -> SaTokenContextModelBox {
        SaTokenContextModelBox::new(
            Arc::clone(&self.request),
            Arc::clone(&self.response),
            Arc::clone(&self.storage),
        )
    }

    fn request(&self) -> Arc<dyn SaRequest> {
        Arc::clone(&self.request)
    }

    fn response(&self) -> Arc<dyn SaResponse> {
        Arc::clone(&self.response)
    }

    fn storage(&self) -> Arc<dyn SaStorage> {
        Arc::clone(&self.storage)
    }
}

/// 账号密码 + RememberMe 登录参数。
#[derive(Debug, Deserialize)]
struct DoLoginQuery {
    #[serde(default)]
    name: String,
    #[serde(default)]
    pwd: String,
    #[serde(default, rename = "rememberMe")]
    remember_me: bool,
}

/// 业务错误映射。
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

/// 将 Header 中的 token 绑定到 DemoContext。
fn bind_token(util: &AsyncStpUtil, token: &str) {
    let name = util.logic().runtime().config().token_name.clone();
    util.logic().runtime().context().storage().set(&name, token);
}

/// 要求已登录并绑定上下文。
async fn require_login(
    util: &AsyncStpUtil,
    headers: &HeaderMap,
) -> Result<(String, String), SaTokenException> {
    let token = token_from_headers(headers)
        .ok_or_else(|| SaTokenException::not_login("未登录", "login"))?;
    let login_id = util
        .get_login_id_by_token(&token)
        .await?
        .ok_or_else(|| SaTokenException::not_login("token 无效", "login"))?;
    bind_token(util, &token);
    Ok((token, login_id))
}

/// 构建 AsyncStpUtil（内存 DAO）。
fn build_util() -> AsyncStpUtil {
    let runtime = AsyncSaTokenRuntime::new(
        Arc::new(SaTokenConfig {
            token_name: "satoken".into(),
            timeout: 2_592_000,
            is_concurrent: true,
            is_share: false,
            is_log: true,
            ..Default::default()
        }),
        Arc::new(SaTokenDaoMemory::new()),
        Arc::new(DemoContext::new()),
    );
    AsyncStpUtil::new("login", Arc::new(runtime))
}

/// 登录 —— `/acc/doLogin`
async fn do_login(
    State(util): State<AsyncStpUtil>,
    Query(q): Query<DoLoginQuery>,
) -> Result<Json<AjaxJson>, AppError> {
    if q.name != "zhang" || q.pwd != "123456" {
        return Ok(Json(AjaxJson::error("登录失败")));
    }

    // Rust 没有 login(id, bool)，用 SaLoginParameter 表达 RememberMe。
    let (param, lasting, timeout) = if q.remember_me {
        (
            SaLoginParameter::create()
                .set_is_lasting_cookie(true)
                .set_timeout(60 * 60 * 24 * 30),
            true,
            60 * 60 * 24 * 30,
        )
    } else {
        (
            SaLoginParameter::create()
                .set_is_lasting_cookie(false)
                .set_timeout(60 * 60 * 2),
            false,
            60 * 60 * 2,
        )
    };

    let token = util.login_with_param("10001", &param).await?;
    Ok(Json(AjaxJson::ok_msg("登录成功").set_data(json!({
        "token": token,
        "rememberMe": lasting,
        "is_lasting_cookie": lasting,
        "timeout": timeout,
    }))))
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

/// Token 信息 —— `/acc/tokenInfo`
async fn token_info(
    State(util): State<AsyncStpUtil>,
    headers: HeaderMap,
) -> Result<Json<AjaxJson>, AppError> {
    let (token, login_id) = require_login(&util, &headers).await?;
    Ok(Json(AjaxJson::ok_data(json!({
        "token_value": token,
        "login_id": login_id,
        "token_name": util.logic().runtime().config().token_name,
    }))))
}

/// 注销 —— `/acc/logout`
async fn logout(
    State(util): State<AsyncStpUtil>,
    headers: HeaderMap,
) -> Result<Json<AjaxJson>, AppError> {
    let (_token, login_id) = require_login(&util, &headers).await?;
    util.logout_by_login_id(&login_id).await?;
    Ok(Json(AjaxJson::ok()))
}

#[tokio::main]
async fn main() {
    let util = build_util();

    let app = Router::new()
        .route("/acc/doLogin", get(do_login).post(do_login))
        .route("/acc/isLogin", get(is_login))
        .route("/acc/tokenInfo", get(token_info))
        .route("/acc/logout", get(logout))
        .with_state(util);

    let addr = "0.0.0.0:8105";
    println!("🚀 Sa-Token-Rs Axum Remember-Me Demo");
    println!("   http://{addr}");
    println!(
        "   说明：Rust 无 login(id, bool)；请用 SaLoginParameter::set_is_lasting_cookie + set_timeout"
    );
    println!("   示例：/acc/doLogin?name=zhang&pwd=123456&rememberMe=true");
    println!("   Header: satoken=<token>");

    let listener = tokio::net::TcpListener::bind(addr).await.expect("bind");
    axum::serve(listener, app).await.expect("serve");
}
