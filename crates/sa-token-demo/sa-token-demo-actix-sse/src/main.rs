//! Sa-Token-Rs Actix SSE Demo
//!
//! 内存 AsyncStpUtil + text/event-stream 推流（需 satoken 头）。

mod util;

use std::sync::Arc;
use std::time::Duration;

use actix_web::web::Bytes;
use actix_web::{App, HttpRequest, HttpResponse, HttpServer, web};
use futures_util::stream;
use sa_token::prelude::{AsyncSaTokenRuntime, AsyncStpUtil, SaTokenConfig, SaTokenDaoMemory};
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

/// 从请求提取 satoken。
fn token_from_req(req: &HttpRequest) -> Option<String> {
    req.headers()
        .get("satoken")
        .and_then(|v| v.to_str().ok())
        .map(str::to_string)
        .or_else(|| {
            req.headers()
                .get(actix_web::http::header::AUTHORIZATION)
                .and_then(|v| v.to_str().ok())
                .and_then(|v| v.strip_prefix("Bearer "))
                .map(str::to_string)
        })
}

/// 校验 Token，返回 login_id。
async fn require_login_id(util: &AsyncStpUtil, req: &HttpRequest) -> Result<String, String> {
    let token = token_from_req(req).ok_or_else(|| "未登录".to_string())?;
    util.get_login_id_by_token(&token)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "token 无效".to_string())
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
async fn do_login(util: web::Data<AsyncStpUtil>, query: web::Query<DoLoginQuery>) -> HttpResponse {
    if query.name == "zhang" && query.pwd == "123456" {
        match util.login("10001").await {
            Ok(token) => HttpResponse::Ok()
                .json(AjaxJson::ok_msg("登录成功").set_data(json!({ "token": token }))),
            Err(e) => HttpResponse::Ok().json(AjaxJson::error(e.to_string())),
        }
    } else {
        HttpResponse::Ok().json(AjaxJson::error("登录失败"))
    }
}

/// SSE 连接 —— `/sse/connect`
///
/// 要求 `satoken` 请求头；校验通过后约每秒推送若干条 `data:` 事件。
async fn sse_connect(util: web::Data<AsyncStpUtil>, req: HttpRequest) -> HttpResponse {
    let login_id = match require_login_id(&util, &req).await {
        Ok(id) => id,
        Err(e) => return HttpResponse::Ok().json(AjaxJson::error(e)),
    };

    let event_stream = stream::unfold(0u32, move |i| {
        let login_id = login_id.clone();
        async move {
            if i >= 5 {
                return None;
            }
            if i > 0 {
                tokio::time::sleep(Duration::from_secs(1)).await;
            }
            let chunk =
                format!("data: {{\"msg\":\"hello\",\"n\":{i},\"loginId\":\"{login_id}\"}}\n\n");
            Some((Ok::<_, actix_web::Error>(Bytes::from(chunk)), i + 1))
        }
    });

    HttpResponse::Ok()
        .content_type("text/event-stream")
        .insert_header(("Cache-Control", "no-cache"))
        .streaming(event_stream)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let util = build_util();
    let addr = ("0.0.0.0", 8112);

    println!("🚀 Sa-Token-Rs Actix SSE Demo");
    println!("   http://{}:{}", addr.0, addr.1);
    println!("   1) GET /acc/doLogin?name=zhang&pwd=123456");
    println!("   2) GET /sse/connect  Header: satoken=<token>");

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(util.clone()))
            .route("/acc/doLogin", web::get().to(do_login))
            .route("/acc/doLogin", web::post().to(do_login))
            .route("/sse/connect", web::get().to(sse_connect))
    })
    .bind(addr)?
    .run()
    .await
}
