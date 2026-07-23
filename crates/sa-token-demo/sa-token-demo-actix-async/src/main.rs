//! Sa-Token-Rs Actix Async Demo
//!
//! 演示 AsyncStpUtil + tokio::spawn：后台登录与无 HTTP 上下文的 token 校验。
//!
//! 对齐 Java `SaTokenContextMockUtil`（非 Web 线程）场景；Rust 侧直接使用
//! token-value API（`login` / `get_login_id_by_token`）+ AsyncStpUtil。

mod util;

use std::sync::Arc;

use actix_web::{App, HttpResponse, HttpServer, web};
use sa_token::prelude::{AsyncSaTokenRuntime, AsyncStpUtil, SaTokenConfig, SaTokenDaoMemory};
use sa_token_core::context::sa_token_context_default_impl::SaTokenContextDefaultImpl;
use sa_token_core::stp::StpInterface;
use serde::Deserialize;
use serde_json::json;
use tokio::sync::{Mutex, oneshot};

use crate::util::AjaxJson;

/// 权限数据源。
struct StpInterfaceImpl;

impl StpInterface for StpInterfaceImpl {
    fn get_permission_list(&self, _login_id: &str, _login_type: &str) -> Vec<String> {
        Vec::new()
    }

    fn get_role_list(&self, _login_id: &str, _login_type: &str) -> Vec<String> {
        Vec::new()
    }
}

/// 应用状态。
#[derive(Clone)]
struct AppState {
    /// 异步工具门面
    util: AsyncStpUtil,
    /// 最近一次后台登录 token
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
        Arc::new(SaTokenContextDefaultImpl),
    )
    .with_stp_interface(Arc::new(StpInterfaceImpl));

    AsyncStpUtil::new("login", Arc::new(runtime))
}

/// 同步登录 —— `/acc/doLogin`
async fn do_login(state: web::Data<AppState>, query: web::Query<DoLoginQuery>) -> HttpResponse {
    if query.name == "zhang" && query.pwd == "123456" {
        match state.util.login("10001").await {
            Ok(token) => HttpResponse::Ok()
                .json(AjaxJson::ok_msg("登录成功").set_data(json!({ "token": token }))),
            Err(e) => HttpResponse::Ok().json(AjaxJson::error(e.to_string())),
        }
    } else {
        HttpResponse::Ok().json(AjaxJson::error("登录失败"))
    }
}

/// 后台线程登录 —— `/async/loginInBackground`
async fn login_in_background(
    state: web::Data<AppState>,
    query: web::Query<BgLoginQuery>,
) -> HttpResponse {
    let util = state.util.clone();
    let login_id = query.id.clone();
    let (tx, rx) = oneshot::channel();

    tokio::spawn(async move {
        let result = util.login(&login_id).await;
        let _ = tx.send(result);
    });

    match rx.await {
        Ok(Ok(token)) => {
            *state.bg_token.lock().await = Some(token.clone());
            HttpResponse::Ok().json(AjaxJson::ok_msg("后台登录完成").set_data(json!({
                "token": token,
                "login_id": query.id,
                "note": "spawned via tokio::spawn; token returned by oneshot",
            })))
        }
        Ok(Err(e)) => HttpResponse::Ok().json(AjaxJson::error(e.to_string())),
        Err(_) => HttpResponse::Ok().json(AjaxJson::error("后台登录通道关闭")),
    }
}

/// 无 HTTP 上下文校验 Token —— `/async/checkWithToken`
async fn check_with_token(
    state: web::Data<AppState>,
    query: web::Query<CheckTokenQuery>,
) -> HttpResponse {
    if query.token.is_empty() {
        return HttpResponse::Ok().json(AjaxJson::error("缺少 token 参数"));
    }
    match state.util.get_login_id_by_token(&query.token).await {
        Ok(Some(id)) => HttpResponse::Ok().json(AjaxJson::ok_data(json!({
            "valid": true,
            "login_id": id,
            "token": query.token,
        }))),
        Ok(None) => HttpResponse::Ok().json(AjaxJson::ok_data(json!({
            "valid": false,
            "token": query.token,
        }))),
        Err(e) => HttpResponse::Ok().json(AjaxJson::error(e.to_string())),
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let state = AppState {
        util: build_util(),
        bg_token: Arc::new(Mutex::new(None)),
    };
    let addr = ("0.0.0.0", 8104);

    println!("🚀 Sa-Token-Rs Actix Async Demo");
    println!("   http://{}:{}", addr.0, addr.1);
    println!("   对齐 Java SaTokenContextMockUtil：Rust 用 token-value API + AsyncStpUtil");

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(state.clone()))
            .route("/acc/doLogin", web::get().to(do_login))
            .route("/acc/doLogin", web::post().to(do_login))
            .route(
                "/async/loginInBackground",
                web::get().to(login_in_background),
            )
            .route("/async/checkWithToken", web::get().to(check_with_token))
    })
    .bind(addr)?
    .run()
    .await
}
