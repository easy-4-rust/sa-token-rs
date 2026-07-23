//! Sa-Token-Rs Axum Device-Lock Demo
//!
//! 对应 Java：`sa-token-demo-device-lock`
//! Rust 无 `isTrustDeviceId` API，用业务层内存信任设备表模拟。

mod util;

use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};

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

/// loginId → 已信任 deviceId 集合。
type TrustStore = Arc<Mutex<HashMap<String, HashSet<String>>>>;

/// 应用状态。
#[derive(Clone)]
struct AppState {
    util: AsyncStpUtil,
    trust: TrustStore,
}

/// 演示用请求上下文。
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

/// 设备登录参数。
#[derive(Debug, Deserialize)]
struct DoLoginQuery {
    #[serde(default)]
    name: String,
    #[serde(default)]
    pwd: String,
    #[serde(default, rename = "deviceId")]
    device_id: String,
    #[serde(default)]
    code: String,
}

/// 信任列表查询。
#[derive(Debug, Deserialize)]
struct LoginIdQuery {
    #[serde(default, rename = "loginId")]
    login_id: String,
}

/// 移除信任设备。
#[derive(Debug, Deserialize)]
struct RemoveDeviceQuery {
    #[serde(default, rename = "loginId")]
    login_id: String,
    #[serde(default, rename = "deviceId")]
    device_id: String,
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

/// 要求已登录。
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

/// 判断设备是否已信任。
fn is_trusted(trust: &TrustStore, login_id: &str, device_id: &str) -> bool {
    trust
        .lock()
        .map(|guard| {
            guard
                .get(login_id)
                .map(|set| set.contains(device_id))
                .unwrap_or(false)
        })
        .unwrap_or(false)
}

/// 将设备加入信任列表。
fn add_trusted(trust: &TrustStore, login_id: &str, device_id: &str) {
    if let Ok(mut guard) = trust.lock() {
        guard
            .entry(login_id.to_string())
            .or_default()
            .insert(device_id.to_string());
    }
}

/// 构建应用状态。
fn build_state() -> AppState {
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
    AppState {
        util: AsyncStpUtil::new("login", Arc::new(runtime)),
        trust: Arc::new(Mutex::new(HashMap::new())),
    }
}

/// 登录（含设备信任校验）—— `/acc/doLogin`
async fn do_login(
    State(state): State<AppState>,
    Query(q): Query<DoLoginQuery>,
) -> Result<Json<AjaxJson>, AppError> {
    if q.name != "zhang" || q.pwd != "123456" {
        return Ok(Json(AjaxJson::error("登录失败")));
    }

    let login_id = "10001";

    // 空 deviceId：允许首次登录（无需验证码）。
    if q.device_id.is_empty() {
        let token = state.util.login(login_id).await?;
        return Ok(Json(
            AjaxJson::ok_msg("登录成功（无设备绑定）")
                .set_data(json!({ "token": token, "loginId": login_id })),
        ));
    }

    // 未信任设备：需要验证码 1234。
    if !is_trusted(&state.trust, login_id, &q.device_id) {
        if q.code != "1234" {
            return Ok(Json(AjaxJson::error(format!(
                "设备 {} 未信任，请提供验证码 code=1234",
                q.device_id
            ))));
        }
        add_trusted(&state.trust, login_id, &q.device_id);
    }

    let param = SaLoginParameter::create().set_device_id(q.device_id.clone());
    let token = state.util.login_with_param(login_id, &param).await?;
    Ok(Json(AjaxJson::ok_msg("登录成功").set_data(json!({
        "token": token,
        "loginId": login_id,
        "deviceId": q.device_id,
    }))))
}

/// 信任设备列表 —— `/device/trustList`
async fn trust_list(
    State(state): State<AppState>,
    Query(q): Query<LoginIdQuery>,
) -> Json<AjaxJson> {
    let login_id = if q.login_id.is_empty() {
        "10001".to_string()
    } else {
        q.login_id
    };
    let devices: Vec<String> = state
        .trust
        .lock()
        .map(|guard| {
            guard
                .get(&login_id)
                .map(|set| set.iter().cloned().collect())
                .unwrap_or_default()
        })
        .unwrap_or_default();
    Json(AjaxJson::ok_data(json!({
        "loginId": login_id,
        "devices": devices,
        "note": "Rust 无 isTrustDeviceId，此为业务层模拟",
    })))
}

/// 移除信任设备 —— `/device/remove`
async fn remove_device(
    State(state): State<AppState>,
    Query(q): Query<RemoveDeviceQuery>,
) -> Json<AjaxJson> {
    if let Ok(mut guard) = state.trust.lock() {
        if let Some(set) = guard.get_mut(&q.login_id) {
            set.remove(&q.device_id);
        }
    }
    Json(AjaxJson::ok_msg(format!(
        "已移除 loginId={} deviceId={}",
        q.login_id, q.device_id
    )))
}

/// 是否登录 —— `/acc/isLogin`
async fn is_login(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<AjaxJson>, AppError> {
    let logged_in = match token_from_headers(&headers) {
        Some(t) => state.util.get_login_id_by_token(&t).await?.is_some(),
        None => false,
    };
    Ok(Json(AjaxJson::ok_msg(format!("是否登录：{logged_in}"))))
}

/// 注销 —— `/acc/logout`
async fn logout(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<AjaxJson>, AppError> {
    let (_token, login_id) = require_login(&state.util, &headers).await?;
    state.util.logout_by_login_id(&login_id).await?;
    Ok(Json(AjaxJson::ok()))
}

#[tokio::main]
async fn main() {
    let state = build_state();

    let app = Router::new()
        .route("/acc/doLogin", get(do_login).post(do_login))
        .route("/acc/isLogin", get(is_login))
        .route("/acc/logout", get(logout))
        .route("/device/trustList", get(trust_list))
        .route("/device/remove", get(remove_device))
        .with_state(state);

    let addr = "0.0.0.0:8107";
    println!("🚀 Sa-Token-Rs Axum Device-Lock Demo");
    println!("   http://{addr}");
    println!("   说明：Rust 无 isTrustDeviceId，使用业务层 HashMap 模拟信任设备");
    println!("   示例：/acc/doLogin?name=zhang&pwd=123456&deviceId=phone-1&code=1234");
    println!("   Header: satoken=<token>");

    let listener = tokio::net::TcpListener::bind(addr).await.expect("bind");
    axum::serve(listener, app).await.expect("serve");
}
