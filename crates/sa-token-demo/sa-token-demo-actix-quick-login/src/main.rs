//! Sa-Token-Rs Actix Quick-Login Demo
//!
//! 对应 Java：`sa-token-demo-quick-login`
//! 未登录返回 HTML 登录页；已登录返回资源页。

mod util;

use std::sync::Arc;

use actix_web::{App, HttpRequest, HttpResponse, HttpServer, web};
use sa_token::prelude::{AsyncSaTokenRuntime, AsyncStpUtil, SaTokenConfig, SaTokenDaoMemory};
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

/// 登录参数。
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

/// 绑定 token 到 DemoContext。
fn bind_token(util: &AsyncStpUtil, token: &str) {
    let name = util.logic().runtime().config().token_name.clone();
    util.logic().runtime().context().storage().set(&name, token);
}

/// 解析当前登录身份（可选）。
async fn current_login(
    util: &AsyncStpUtil,
    req: &HttpRequest,
) -> Result<Option<(String, String)>, String> {
    let Some(token) = token_from_req(req) else {
        return Ok(None);
    };
    match util
        .get_login_id_by_token(&token)
        .await
        .map_err(|e| e.to_string())?
    {
        Some(login_id) => {
            bind_token(util, &token);
            Ok(Some((token, login_id)))
        }
        None => Ok(None),
    }
}

/// HTML 登录表单。
fn login_page_html() -> HttpResponse {
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(
            r#"<!DOCTYPE html>
<html lang="zh-CN">
<head><meta charset="utf-8"><title>Quick Login</title></head>
<body>
  <h1>请登录</h1>
  <form method="get" action="/doLogin">
    <p>账号：<input name="name" value="zhang"></p>
    <p>密码：<input name="pwd" type="password" value="123456"></p>
    <button type="submit">登录</button>
  </form>
</body>
</html>"#,
        )
}

/// 已登录资源页 HTML。
fn resource_page_html(login_id: &str) -> HttpResponse {
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(format!(
            r#"<!DOCTYPE html>
<html lang="zh-CN">
<head><meta charset="utf-8"><title>已登录</title></head>
<body>
  <h1>已登录资源页</h1>
  <p>loginId: {login_id}</p>
  <p><a href="/res">进入受保护资源 /res</a></p>
</body>
</html>"#
        ))
}

/// 构建 AsyncStpUtil。
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

/// 首页：未登录 → 登录页；已登录 → 资源页。
async fn index(util: web::Data<AsyncStpUtil>, req: HttpRequest) -> HttpResponse {
    match current_login(&util, &req).await {
        Ok(Some((_token, login_id))) => resource_page_html(&login_id),
        Ok(None) => login_page_html(),
        Err(e) => HttpResponse::Ok().json(AjaxJson::error(e)),
    }
}

/// 登录 —— `/doLogin`
async fn do_login(util: web::Data<AsyncStpUtil>, query: web::Query<DoLoginQuery>) -> HttpResponse {
    if query.name != "zhang" || query.pwd != "123456" {
        return HttpResponse::Ok().json(AjaxJson::error("登录失败"));
    }
    match util.login("10001").await {
        Ok(token) => HttpResponse::Ok().json(
            AjaxJson::ok_msg("登录成功，请在后续请求 Header 携带 satoken，并访问 /").set_data(
                json!({
                    "token": token,
                    "redirect": "/",
                    "hint": "Header: satoken=<token>",
                }),
            ),
        ),
        Err(e) => HttpResponse::Ok().json(AjaxJson::error(e.to_string())),
    }
}

/// 受保护资源 —— `/res`
async fn res(util: web::Data<AsyncStpUtil>, req: HttpRequest) -> HttpResponse {
    match current_login(&util, &req).await {
        Ok(Some((_token, login_id))) => HttpResponse::Ok().json(AjaxJson::ok_data(json!({
            "page": "protected",
            "loginId": login_id,
            "msg": "已登录资源",
        }))),
        Ok(None) => login_page_html(),
        Err(e) => HttpResponse::Ok().json(AjaxJson::error(e)),
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let util = build_util();
    let addr = ("0.0.0.0", 8110);

    println!("🚀 Sa-Token-Rs Actix Quick-Login Demo");
    println!("   http://{}:{}", addr.0, addr.1);
    println!("   未登录访问 / 返回 HTML 登录页；登录后 Header 带 satoken 再访问 /");
    println!("   账号：zhang / 123456");

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(util.clone()))
            .route("/", web::get().to(index))
            .route("/doLogin", web::get().to(do_login))
            .route("/doLogin", web::post().to(do_login))
            .route("/res", web::get().to(res))
    })
    .bind(addr)?
    .run()
    .await
}
