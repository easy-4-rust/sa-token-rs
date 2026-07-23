//! Sa-Token-Rs Axum Async Demo
//!
//! 演示 AsyncStpUtil + tokio::spawn：后台登录与无 HTTP 上下文的 token 校验。
//!
//! 对齐 Java `SaTokenContextMockUtil`（非 Web 线程）场景；Rust 侧直接使用
//! token-value API（`login` / `get_login_id_by_token`）+ AsyncStpUtil。

mod util;

use std::sync::Arc;

use axum::Json;
use axum::Router;
use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use sa_token::prelude::{
    AsyncSaTokenRuntime, AsyncStpUtil, SaTokenConfig, SaTokenDaoMemory, SaTokenException,
};
use sa_token_core::context::sa_token_context_default_impl::SaTokenContextDefaultImpl;
use sa_token_core::stp::StpInterface;
use serde::Deserialize;
use serde_json::json;
use tokio::sync::{Mutex, oneshot};

use crate::util::AjaxJson;

/// 权限数据源（本 Demo 仅演示登录，权限列表可为空）。
struct StpInterfaceImpl;

impl StpInterface for StpInterfaceImpl {
    fn get_permission_list(&self, _login_id: &str, _login_type: &str) -> Vec<String> {
        Vec::new()
    }

    fn get_role_list(&self, _login_id: &str, _login_type: &str) -> Vec<String> {
        Vec::new()
    }
}

/// 应用状态：AsyncStpUtil + 后台登录结果槽。
#[derive(Clone)]
struct AppState {
    /// 异步工具门面
    util: AsyncStpUtil,
    /// 最近一次后台登录产生的 token（亦可改用 oneshot）
    bg_token: Arc<Mutex<Option<String>>>,
}

/// 账号密码登录参数。
#[derive(Debug, Deserialize)]
struct DoLoginQuery {
    #[serde(default)]
    name: String,
    #[serde(default)]
    pwd: String,
}

/// 后台登录参数。
#[derive(Debug, Deserialize)]
struct BgLoginQuery {
    #[serde(default = "default_id")]
    id: String,
}

fn default_id() -> String {
    "10001".into()
}

/// 按 token 校验参数。
#[derive(Debug, Deserialize)]
struct CheckTokenQuery {
    #[serde(default)]
    token: String,
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

/// 构建运行时。
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
        Arc::new(SaTokenContextDefaultImpl),
    )
    .with_stp_interface(Arc::new(StpInterfaceImpl));

    AsyncStpUtil::new("login", Arc::new(runtime))
}

/// 同步登录 —— `/acc/doLogin`
async fn do_login(
    State(state): State<AppState>,
    Query(q): Query<DoLoginQuery>,
) -> Result<Json<AjaxJson>, AppError> {
    if q.name == "zhang" && q.pwd == "123456" {
        let token = state.util.login("10001").await?;
        return Ok(Json(
            AjaxJson::ok_msg("登录成功").set_data(json!({ "token": token })),
        ));
    }
    Ok(Json(AjaxJson::error("登录失败")))
}

/// 后台线程登录 —— `/async/loginInBackground`
///
/// 在 `tokio::spawn` 中调用 `util.login`，通过 oneshot 回传 token，
/// 并写入共享槽位，模拟非 Web 线程完成登录。
async fn login_in_background(
    State(state): State<AppState>,
    Query(q): Query<BgLoginQuery>,
) -> Result<Json<AjaxJson>, AppError> {
    let util = state.util.clone();
    let login_id = q.id.clone();
    let (tx, rx) = oneshot::channel();

    tokio::spawn(async move {
        let result = util.login(&login_id).await;
        let _ = tx.send(result);
    });

    let token = rx
        .await
        .map_err(|_| SaTokenException::other("后台登录通道关闭"))??;

    *state.bg_token.lock().await = Some(token.clone());

    Ok(Json(AjaxJson::ok_msg("后台登录完成").set_data(json!({
        "token": token,
        "login_id": q.id,
        "note": "spawned via tokio::spawn; token returned by oneshot",
    }))))
}

/// 无 HTTP 上下文校验 Token —— `/async/checkWithToken`
///
/// 仅依赖 token 值 API，不读 Cookie / Header 上下文。
async fn check_with_token(
    State(state): State<AppState>,
    Query(q): Query<CheckTokenQuery>,
) -> Result<Json<AjaxJson>, AppError> {
    if q.token.is_empty() {
        return Ok(Json(AjaxJson::error("缺少 token 参数")));
    }
    let login_id = state.util.get_login_id_by_token(&q.token).await?;
    match login_id {
        Some(id) => Ok(Json(AjaxJson::ok_data(json!({
            "valid": true,
            "login_id": id,
            "token": q.token,
        })))),
        None => Ok(Json(AjaxJson::ok_data(json!({
            "valid": false,
            "token": q.token,
        })))),
    }
}

#[tokio::main]
async fn main() {
    let state = AppState {
        util: build_util(),
        bg_token: Arc::new(Mutex::new(None)),
    };

    let app = Router::new()
        .route("/acc/doLogin", get(do_login).post(do_login))
        .route("/async/loginInBackground", get(login_in_background))
        .route("/async/checkWithToken", get(check_with_token))
        .with_state(state);

    let addr = "0.0.0.0:8103";
    println!("🚀 Sa-Token-Rs Axum Async Demo");
    println!("   http://{addr}");
    println!("   对齐 Java SaTokenContextMockUtil：Rust 用 token-value API + AsyncStpUtil");

    let listener = tokio::net::TcpListener::bind(addr).await.expect("bind");
    axum::serve(listener, app).await.expect("serve");
}
