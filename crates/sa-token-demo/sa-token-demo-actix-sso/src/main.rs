//! Sa-Token-Rs Actix SSO Server Mini Demo
//!
//! 对应 Java：`sa-token-demo-sso-server`（精简版，AsyncStpUtil + actix-web）

mod util;

use std::future::Future;
use std::sync::Arc;

use actix_web::{App, HttpRequest, HttpResponse, HttpServer, web};
use sa_token::prelude::{AsyncSaTokenRuntime, AsyncStpUtil, SaTokenConfig};
use sa_token_core::context::sa_token_context_default_impl::SaTokenContextDefaultImpl;
use sa_token_core::dao::async_sa_token_dao::AsyncSaTokenDao;
use sa_token_core::dao::sa_token_dao::SaTokenDao;
use sa_token_dao_memory::SaTokenDaoMemory;
use sa_token_sign::sign::SaSignConfig;
use sa_token_sso::sso::config::{SaSsoClientModel, SaSsoServerConfig};
use sa_token_sso::sso::exception::SaSsoException;
use sa_token_sso::sso::strategy::SaSsoServerStrategy;
use sa_token_sso::sso::template::{SaSsoServerAuth, SaSsoServerTemplate};
use serde::Deserialize;
use serde_json::{Value, json};

use crate::util::AjaxJson;

/// 在已有 Tokio runtime 上阻塞执行异步 Future（供同步 Auth 端口使用）。
fn block_on<T>(fut: impl Future<Output = T>) -> T {
    tokio::task::block_in_place(|| tokio::runtime::Handle::current().block_on(fut))
}

/// 将 JSON loginId 转为字符串。
fn value_as_login_id(login_id: &Value) -> String {
    match login_id {
        Value::String(s) => s.clone(),
        Value::Number(n) => n.to_string(),
        other => other.to_string(),
    }
}

/// 从请求中读取 satoken。
fn extract_token(req: &HttpRequest) -> Option<String> {
    req.headers()
        .get("satoken")
        .and_then(|v| v.to_str().ok())
        .map(str::to_string)
        .or_else(|| req.cookie("satoken").map(|c| c.value().to_string()))
        .or_else(|| {
            req.headers()
                .get(actix_web::http::header::AUTHORIZATION)
                .and_then(|v| v.to_str().ok())
                .and_then(|v| v.strip_prefix("Bearer "))
                .map(str::to_string)
        })
}

/// 基于 AsyncStpUtil 的 SSO Server Auth 端口。
struct DemoServerAuth {
    util: AsyncStpUtil,
}

impl SaSsoServerAuth for DemoServerAuth {
    /// 按 Token 查询登录设备 ID。
    fn login_device_id_by_token(
        &self,
        token_value: &str,
    ) -> Result<Option<String>, SaSsoException> {
        let util = self.util.clone();
        let token = token_value.to_string();
        block_on(async move {
            util.logic()
                .get_login_device_id_by_token(&token)
                .await
                .map_err(|e| SaSsoException::new(0, e.to_string()))
        })
    }

    /// 按 Token 查询剩余有效期（秒）。
    fn token_timeout(&self, token_value: &str) -> Result<i64, SaSsoException> {
        let util = self.util.clone();
        let token = token_value.to_string();
        block_on(async move {
            util.logic()
                .get_token_timeout_by_token(&token)
                .await
                .map_err(|e| SaSsoException::new(0, e.to_string()))
        })
    }

    /// 按 loginId 查询 Account-Session 剩余有效期（秒）。
    fn session_timeout(&self, login_id: &Value) -> Result<i64, SaSsoException> {
        let util = self.util.clone();
        let id = value_as_login_id(login_id);
        block_on(async move {
            let session = util
                .get_session_by_login_id(&id)
                .await
                .map_err(|e| SaSsoException::new(0, e.to_string()))?;
            util.logic()
                .runtime()
                .dao()
                .get_session_timeout(session.id())
                .await
                .map_err(|e| SaSsoException::new(0, e.to_string()))
        })
    }

    /// 注销指定账号会话。
    fn logout(&self, login_id: &Value, _device_id: Option<String>) -> Result<(), SaSsoException> {
        let util = self.util.clone();
        let id = value_as_login_id(login_id);
        block_on(async move {
            util.logout_by_login_id(&id)
                .await
                .map_err(|e| SaSsoException::new(0, e.to_string()))
        })
    }
}

/// 应用状态。
#[derive(Clone)]
struct AppState {
    util: AsyncStpUtil,
    template: Arc<SaSsoServerTemplate>,
}

#[derive(Debug, Deserialize)]
struct LoginQuery {
    #[serde(default)]
    name: String,
    #[serde(default)]
    pwd: String,
}

#[derive(Debug, Deserialize)]
struct AuthQuery {
    #[serde(default = "default_client")]
    client: String,
    #[serde(default)]
    redirect: String,
}

fn default_client() -> String {
    "sso-client".into()
}

#[derive(Debug, Deserialize)]
struct CheckTicketQuery {
    ticket: String,
    #[serde(default = "default_client")]
    client: String,
}

/// 初始化 SSO Server 与 AsyncStpUtil（共享内存 DAO）。
fn build_state() -> AppState {
    let dao = Arc::new(SaTokenDaoMemory::new());
    let runtime = AsyncSaTokenRuntime::new(
        Arc::new(SaTokenConfig::default()),
        Arc::clone(&dao) as Arc<dyn AsyncSaTokenDao>,
        Arc::new(SaTokenContextDefaultImpl),
    );
    let util = AsyncStpUtil::new("login", Arc::new(runtime));

    let mut config = SaSsoServerConfig {
        is_check_sign: false,
        allow_anon_client: true,
        allow_url: "*".into(),
        home_route: Some("/".into()),
        ..Default::default()
    };
    config.add_client(SaSsoClientModel {
        client: "sso-client".into(),
        allow_url: "*".into(),
        ..Default::default()
    });

    let util_for_strategy = util.clone();
    let mut strategy = SaSsoServerStrategy::default();
    // 账号密码登录策略：sa / 123456 → 登录 10001。
    strategy.do_login_handle = Arc::new(move |name, pwd| {
        if name == "sa" && pwd == "123456" {
            let util = util_for_strategy.clone();
            match block_on(async move { util.login("10001").await }) {
                Ok(token) => json!({"code": 200, "msg": "ok", "data": {"satoken": token}}),
                Err(e) => json!({"code": 500, "msg": e.to_string()}),
            }
        } else {
            json!({"code": 500, "msg": "账号名或密码错误"})
        }
    });

    let template = SaSsoServerTemplate::new(
        Arc::new(config),
        Arc::new(strategy),
        Arc::new(SaSignConfig::default()),
        Arc::clone(&dao) as Arc<dyn SaTokenDao>,
        Arc::new(DemoServerAuth { util: util.clone() }),
        "satoken",
    )
    .expect("sso server template");

    AppState {
        util,
        template: Arc::new(template),
    }
}

/// SSO 登录 —— `/sso/doLogin`
async fn do_login(state: web::Data<AppState>, query: web::Query<LoginQuery>) -> HttpResponse {
    let result = (state.template.strategy.do_login_handle)(&query.name, &query.pwd);
    HttpResponse::Ok().json(result)
}

/// 授权下发 ticket —— `/sso/auth`
async fn sso_auth(
    state: web::Data<AppState>,
    req: HttpRequest,
    query: web::Query<AuthQuery>,
) -> HttpResponse {
    let Some(token) = extract_token(&req) else {
        return HttpResponse::Ok().json(AjaxJson::error("未登录，请先调用 /sso/doLogin"));
    };
    let Ok(Some(login_id)) = state.util.get_login_id_by_token(&token).await else {
        return HttpResponse::Ok().json(AjaxJson::error("未登录，请先调用 /sso/doLogin"));
    };
    if query.redirect.is_empty() {
        return HttpResponse::Ok().json(AjaxJson::error("缺少 redirect 参数"));
    }
    match state
        .template
        .build_redirect_url(&query.client, &query.redirect, json!(login_id), &token)
    {
        Ok(url) => HttpResponse::Ok().json(AjaxJson::ok_data(json!({ "redirect": url }))),
        Err(e) => HttpResponse::Ok().json(AjaxJson::error(e.to_string())),
    }
}

/// 校验并消费 ticket —— `/sso/checkTicket`
async fn check_ticket(
    state: web::Data<AppState>,
    query: web::Query<CheckTicketQuery>,
) -> HttpResponse {
    match state
        .template
        .check_ticket_and_delete(&query.ticket, &query.client)
    {
        Ok(model) => HttpResponse::Ok().json(AjaxJson::ok_data(model.login_id)),
        Err(e) => HttpResponse::Ok().json(AjaxJson::error(e.to_string())),
    }
}

/// 是否已登录 —— `/acc/isLogin`
async fn is_login(state: web::Data<AppState>, req: HttpRequest) -> HttpResponse {
    let Some(token) = extract_token(&req) else {
        return HttpResponse::Ok().json(AjaxJson::ok_data(false));
    };
    match state.util.get_login_id_by_token(&token).await {
        Ok(Some(_)) => HttpResponse::Ok().json(AjaxJson::ok_data(true)),
        Ok(None) => HttpResponse::Ok().json(AjaxJson::ok_data(false)),
        Err(e) => HttpResponse::Ok().json(AjaxJson::error(e.to_string())),
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let state = build_state();
    let addr = ("0.0.0.0", 8096);
    println!("🚀 Sa-Token-Rs Actix SSO Server Demo");
    println!("   http://{}:{}", addr.0, addr.1);
    println!("   client=sso-client  login: sa / 123456");

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(state.clone()))
            .app_data(web::Data::new(state.util.clone()))
            .route("/sso/doLogin", web::get().to(do_login))
            .route("/sso/auth", web::get().to(sso_auth))
            .route("/sso/checkTicket", web::get().to(check_ticket))
            .route("/acc/isLogin", web::get().to(is_login))
    })
    .bind(addr)?
    .run()
    .await
}
