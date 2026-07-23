//! Sa-Token-Rs Axum Case Demo
//!
//! 对应 Java：`sa-token-demo-case`
//! 使用 AsyncStpUtil + SaTokenDaoMemory（与 redis demo 相同异步路径）。

mod util;

use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

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
use sa_token_core::stp::StpInterface;
use serde::Deserialize;
use serde_json::json;

use crate::util::AjaxJson;

/// 演示用请求上下文：始终有效，便于 open_safe / check_role 等依赖当前 Token 的 API。
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

/// 权限 / 角色数据源。
struct StpInterfaceImpl;

impl StpInterface for StpInterfaceImpl {
    fn get_permission_list(&self, _login_id: &str, _login_type: &str) -> Vec<String> {
        vec!["user-add".into(), "user-delete".into(), "101".into()]
    }

    fn get_role_list(&self, _login_id: &str, _login_type: &str) -> Vec<String> {
        vec!["admin".into(), "super-admin".into()]
    }
}

/// 账号密码登录参数。
#[derive(Debug, Deserialize)]
struct DoLoginQuery {
    #[serde(default)]
    name: String,
    #[serde(default)]
    pwd: String,
}

/// 登录 id 参数。
#[derive(Debug, Deserialize)]
struct IdQuery {
    #[serde(default = "default_id")]
    id: String,
}

fn default_id() -> String {
    "10001".into()
}

/// 封禁参数。
#[derive(Debug, Deserialize)]
struct DisableQuery {
    #[serde(default = "default_id")]
    id: String,
    #[serde(default = "default_disable_time")]
    time: i64,
}

fn default_disable_time() -> i64 {
    86_400
}

/// 二级认证参数。
#[derive(Debug, Deserialize)]
struct SafeQuery {
    #[serde(default = "default_safe_time")]
    safe_time: i64,
}

fn default_safe_time() -> i64 {
    120
}

/// 临时 Token 参数。
#[derive(Debug, Deserialize)]
struct TempTokenQuery {
    #[serde(default)]
    value: String,
    #[serde(default = "default_temp_timeout")]
    timeout: i64,
}

fn default_temp_timeout() -> i64 {
    60
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

/// 将 Header 中的 token 绑定到当前 DemoContext，供上下文相关 API 使用。
fn bind_token(util: &AsyncStpUtil, token: &str) {
    let name = util.logic().runtime().config().token_name.clone();
    util.logic().runtime().context().storage().set(&name, token);
}

/// 要求已登录并绑定上下文，返回 (token, login_id)。
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
    )
    .with_stp_interface(Arc::new(StpInterfaceImpl));

    AsyncStpUtil::new("login", Arc::new(runtime))
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
    let (_token, login_id) = require_login(&util, &headers).await?;
    util.logout_by_login_id(&login_id).await?;
    Ok(Json(AjaxJson::ok()))
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

/// 测试登录 —— `/test/login`
async fn test_login(
    State(util): State<AsyncStpUtil>,
    Query(q): Query<IdQuery>,
) -> Result<Json<AjaxJson>, AppError> {
    let token = util.login(&q.id).await?;
    Ok(Json(AjaxJson::ok().set_data(json!({ "token": token }))))
}

/// 测试注销 —— `/test/logout`
async fn test_logout(
    State(util): State<AsyncStpUtil>,
    headers: HeaderMap,
) -> Result<Json<AjaxJson>, AppError> {
    let (_token, login_id) = require_login(&util, &headers).await?;
    util.logout_by_login_id(&login_id).await?;
    Ok(Json(AjaxJson::ok()))
}

/// 角色校验 —— `/test/testRole`
async fn test_role(
    State(util): State<AsyncStpUtil>,
    headers: HeaderMap,
) -> Result<Json<AjaxJson>, AppError> {
    require_login(&util, &headers).await?;
    util.check_role("admin").await?;
    Ok(Json(AjaxJson::ok_msg("角色测试通过")))
}

/// 权限校验 —— `/test/testJur`
async fn test_jur(
    State(util): State<AsyncStpUtil>,
    headers: HeaderMap,
) -> Result<Json<AjaxJson>, AppError> {
    require_login(&util, &headers).await?;
    util.check_permission("user-add").await?;
    Ok(Json(AjaxJson::ok_msg("权限测试通过")))
}

/// Account-Session —— `/test/session`
async fn test_session(
    State(util): State<AsyncStpUtil>,
    headers: HeaderMap,
) -> Result<Json<AjaxJson>, AppError> {
    let (_token, login_id) = require_login(&util, &headers).await?;
    let mut session = util.get_session_by_login_id(&login_id).await?;
    let prev = session.get("name").cloned();
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    session.set("name", json!(now));
    util.logic()
        .runtime()
        .dao()
        .update_session(&session)
        .await?;
    Ok(Json(AjaxJson::ok_data(json!({
        "session_id": session.id(),
        "prev_name": prev,
        "name": session.get("name"),
    }))))
}

/// 踢人下线 —— `/test/kickOut`
async fn kick_out(
    State(util): State<AsyncStpUtil>,
    Query(q): Query<IdQuery>,
) -> Result<Json<AjaxJson>, AppError> {
    util.kickout_by_login_id(&q.id).await?;
    Ok(Json(AjaxJson::ok_msg(format!("已踢下线：{}", q.id))))
}

/// 开启二级认证 —— `/test/openSafe`
async fn open_safe(
    State(util): State<AsyncStpUtil>,
    headers: HeaderMap,
    Query(q): Query<SafeQuery>,
) -> Result<Json<AjaxJson>, AppError> {
    require_login(&util, &headers).await?;
    util.open_safe(q.safe_time).await?;
    Ok(Json(AjaxJson::ok_msg(format!(
        "已开启二级认证，有效期 {} 秒",
        q.safe_time
    ))))
}

/// 校验二级认证 —— `/test/checkSafe`
async fn check_safe(
    State(util): State<AsyncStpUtil>,
    headers: HeaderMap,
) -> Result<Json<AjaxJson>, AppError> {
    require_login(&util, &headers).await?;
    util.logic().check_safe().await?;
    let is_safe = util.is_safe().await?;
    Ok(Json(AjaxJson::ok_msg(format!(
        "二级认证通过，is_safe={is_safe}"
    ))))
}

/// 封禁账号 —— `/test/disable`
async fn disable(
    State(util): State<AsyncStpUtil>,
    Query(q): Query<DisableQuery>,
) -> Result<Json<AjaxJson>, AppError> {
    util.disable(&q.id, q.time).await?;
    Ok(Json(AjaxJson::ok_msg(format!(
        "已封禁 {}，时长 {} 秒",
        q.id, q.time
    ))))
}

/// 是否封禁 —— `/test/isDisable`
async fn is_disable(
    State(util): State<AsyncStpUtil>,
    Query(q): Query<IdQuery>,
) -> Result<Json<AjaxJson>, AppError> {
    let disabled = util.is_disable(&q.id).await?;
    Ok(Json(AjaxJson::ok_msg(format!(
        "账号 {} 是否封禁：{disabled}",
        q.id
    ))))
}

/// 身份切换 —— `/test/switchTo`
async fn switch_to(
    State(util): State<AsyncStpUtil>,
    headers: HeaderMap,
    Query(q): Query<IdQuery>,
) -> Result<Json<AjaxJson>, AppError> {
    require_login(&util, &headers).await?;
    util.switch_to(&q.id).await?;
    Ok(Json(AjaxJson::ok_msg(format!(
        "已切换到 {}，is_switch={}",
        q.id,
        util.is_switch()
    ))))
}

/// 结束身份切换 —— `/test/endSwitch`
async fn end_switch(
    State(util): State<AsyncStpUtil>,
    headers: HeaderMap,
) -> Result<Json<AjaxJson>, AppError> {
    require_login(&util, &headers).await?;
    util.end_switch();
    Ok(Json(AjaxJson::ok_msg(format!(
        "已结束切换，is_switch={}",
        util.is_switch()
    ))))
}

/// 临时 Token 演示 —— `/test/tempToken`
///
/// 注：Java `SaTempUtil` 依赖全局 `SaManager`；本 Demo 走 Async DAO 路径，
/// 用随机 token + TTL 模拟临时凭证（未引入同步 SaManager 初始化）。
async fn temp_token(
    State(util): State<AsyncStpUtil>,
    Query(q): Query<TempTokenQuery>,
) -> Result<Json<AjaxJson>, AppError> {
    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);
    let token = format!("temp_{millis:x}");
    let key = format!("satoken:temp:{token}");
    let value = if q.value.is_empty() {
        "demo-temp-value".to_string()
    } else {
        q.value.clone()
    };
    util.logic()
        .runtime()
        .dao()
        .set(&key, &value, q.timeout)
        .await?;
    Ok(Json(AjaxJson::ok_data(json!({
        "temp_token": token,
        "value": value,
        "timeout": q.timeout,
        "note": "Async DAO demo; SaTempUtil needs SaManager sync init",
    }))))
}

/// lasting cookie + timeout 登录 —— `/test/loginByParam`
async fn login_by_param(
    State(util): State<AsyncStpUtil>,
    Query(q): Query<IdQuery>,
) -> Result<Json<AjaxJson>, AppError> {
    let param = SaLoginParameter::create()
        .set_is_lasting_cookie(true)
        .set_timeout(2_592_000);
    let token = util.login_with_param(&q.id, &param).await?;
    Ok(Json(AjaxJson::ok().set_data(json!({
        "token": token,
        "is_lasting_cookie": true,
        "timeout": 2_592_000,
    }))))
}

#[tokio::main]
async fn main() {
    let util = build_util();

    let app = Router::new()
        .route("/acc/doLogin", get(do_login).post(do_login))
        .route("/acc/isLogin", get(is_login))
        .route("/acc/logout", get(logout))
        .route("/acc/tokenInfo", get(token_info))
        .route("/test/login", get(test_login))
        .route("/test/logout", get(test_logout))
        .route("/test/testRole", get(test_role))
        .route("/test/testJur", get(test_jur))
        .route("/test/session", get(test_session))
        .route("/test/kickOut", get(kick_out))
        .route("/test/openSafe", get(open_safe))
        .route("/test/checkSafe", get(check_safe))
        .route("/test/disable", get(disable))
        .route("/test/isDisable", get(is_disable))
        .route("/test/switchTo", get(switch_to))
        .route("/test/endSwitch", get(end_switch))
        .route("/test/tempToken", get(temp_token))
        .route("/test/loginByParam", get(login_by_param))
        .with_state(util);

    let addr = "0.0.0.0:8101";
    println!("🚀 Sa-Token-Rs Axum Case Demo（对应 sa-token-demo-case）");
    println!("   http://{addr}");
    println!("   Header: satoken=<token>");

    let listener = tokio::net::TcpListener::bind(addr).await.expect("bind");
    axum::serve(listener, app).await.expect("serve");
}
