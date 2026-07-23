//! Sa-Token-Rs Actix Remember-Me Demo
//!
//! 对应 Java：`sa-token-demo-remember-me`
//! Rust 无 `login(id, bool)`，请用 `SaLoginParameter::set_is_lasting_cookie`。

mod util;

use std::sync::Arc;

use actix_web::{App, HttpRequest, HttpResponse, HttpServer, web};
use sa_token::prelude::{
    AsyncSaTokenRuntime, AsyncStpUtil, SaLoginParameter, SaTokenConfig, SaTokenDaoMemory,
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

/// 要求已登录。
async fn require_login(util: &AsyncStpUtil, req: &HttpRequest) -> Result<(String, String), String> {
    let token = token_from_req(req).ok_or_else(|| "未登录".to_string())?;
    let login_id = util
        .get_login_id_by_token(&token)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "token 无效".to_string())?;
    bind_token(util, &token);
    Ok((token, login_id))
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

/// 登录 —— `/acc/doLogin`
async fn do_login(util: web::Data<AsyncStpUtil>, query: web::Query<DoLoginQuery>) -> HttpResponse {
    if query.name != "zhang" || query.pwd != "123456" {
        return HttpResponse::Ok().json(AjaxJson::error("登录失败"));
    }

    // Rust 没有 login(id, bool)，用 SaLoginParameter 表达 RememberMe。
    let (param, lasting, timeout) = if query.remember_me {
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

    match util.login_with_param("10001", &param).await {
        Ok(token) => HttpResponse::Ok().json(AjaxJson::ok_msg("登录成功").set_data(json!({
            "token": token,
            "rememberMe": lasting,
            "is_lasting_cookie": lasting,
            "timeout": timeout,
        }))),
        Err(e) => HttpResponse::Ok().json(AjaxJson::error(e.to_string())),
    }
}

/// 是否登录 —— `/acc/isLogin`
async fn is_login(util: web::Data<AsyncStpUtil>, req: HttpRequest) -> HttpResponse {
    let logged_in = match token_from_req(&req) {
        Some(t) => util
            .get_login_id_by_token(&t)
            .await
            .ok()
            .flatten()
            .is_some(),
        None => false,
    };
    HttpResponse::Ok().json(AjaxJson::ok_msg(format!("是否登录：{logged_in}")))
}

/// Token 信息 —— `/acc/tokenInfo`
async fn token_info(util: web::Data<AsyncStpUtil>, req: HttpRequest) -> HttpResponse {
    match require_login(&util, &req).await {
        Ok((token, login_id)) => HttpResponse::Ok().json(AjaxJson::ok_data(json!({
            "token_value": token,
            "login_id": login_id,
            "token_name": util.logic().runtime().config().token_name,
        }))),
        Err(e) => HttpResponse::Ok().json(AjaxJson::error(e)),
    }
}

/// 注销 —— `/acc/logout`
async fn logout(util: web::Data<AsyncStpUtil>, req: HttpRequest) -> HttpResponse {
    match require_login(&util, &req).await {
        Ok((_token, login_id)) => match util.logout_by_login_id(&login_id).await {
            Ok(()) => HttpResponse::Ok().json(AjaxJson::ok()),
            Err(e) => HttpResponse::Ok().json(AjaxJson::error(e.to_string())),
        },
        Err(e) => HttpResponse::Ok().json(AjaxJson::error(e)),
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let util = build_util();
    let addr = ("0.0.0.0", 8106);

    println!("🚀 Sa-Token-Rs Actix Remember-Me Demo");
    println!("   http://{}:{}", addr.0, addr.1);
    println!(
        "   说明：Rust 无 login(id, bool)；请用 SaLoginParameter::set_is_lasting_cookie + set_timeout"
    );
    println!("   示例：/acc/doLogin?name=zhang&pwd=123456&rememberMe=true");
    println!("   Header: satoken=<token>");

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(util.clone()))
            .route("/acc/doLogin", web::get().to(do_login))
            .route("/acc/doLogin", web::post().to(do_login))
            .route("/acc/isLogin", web::get().to(is_login))
            .route("/acc/tokenInfo", web::get().to(token_info))
            .route("/acc/logout", web::get().to(logout))
    })
    .bind(addr)?
    .run()
    .await
}
