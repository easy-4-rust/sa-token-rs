//! Sa-Token-Rs Axum SSE Demo
//!
//! 内存 AsyncStpUtil + Server-Sent Events 推流（需 satoken 头）。

mod util;

use std::convert::Infallible;
use std::sync::Arc;
use std::time::Duration;

use axum::Json;
use axum::Router;
use axum::extract::{Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::sse::{Event, KeepAlive, Sse};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use futures_util::stream::{self, Stream};
use sa_token::prelude::{
    AsyncSaTokenRuntime, AsyncStpUtil, SaTokenConfig, SaTokenDaoMemory, SaTokenException,
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

/// 校验 Token，返回 login_id。
async fn require_login_id(
    util: &AsyncStpUtil,
    headers: &HeaderMap,
) -> Result<String, SaTokenException> {
    let token = token_from_headers(headers)
        .ok_or_else(|| SaTokenException::not_login("未登录", "login"))?;
    util.get_login_id_by_token(&token)
        .await?
        .ok_or_else(|| SaTokenException::not_login("token 无效", "login"))
}

/// 构建内存 AsyncStpUtil。
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

/// SSE 连接 —— `/sse/connect`
///
/// 要求 `satoken` 请求头；校验通过后约每秒推送若干条事件。
async fn sse_connect(
    State(util): State<AsyncStpUtil>,
    headers: HeaderMap,
) -> Result<Sse<impl Stream<Item = Result<Event, Infallible>>>, AppError> {
    let login_id = require_login_id(&util, &headers).await?;

    let stream = stream::unfold(0u32, move |i| {
        let login_id = login_id.clone();
        async move {
            if i >= 5 {
                return None;
            }
            if i > 0 {
                tokio::time::sleep(Duration::from_secs(1)).await;
            }
            let payload = format!(r#"{{"msg":"hello","n":{i},"loginId":"{login_id}"}}"#);
            let event = Event::default().data(payload);
            Some((Ok(event), i + 1))
        }
    });

    Ok(Sse::new(stream).keep_alive(KeepAlive::default()))
}

#[tokio::main]
async fn main() {
    let util = build_util();

    let app = Router::new()
        .route("/acc/doLogin", get(do_login).post(do_login))
        .route("/sse/connect", get(sse_connect))
        .with_state(util);

    let addr = "0.0.0.0:8111";
    println!("🚀 Sa-Token-Rs Axum SSE Demo");
    println!("   http://{addr}");
    println!("   1) GET /acc/doLogin?name=zhang&pwd=123456");
    println!("   2) GET /sse/connect  Header: satoken=<token>");

    let listener = tokio::net::TcpListener::bind(addr).await.expect("bind");
    axum::serve(listener, app).await.expect("serve");
}
