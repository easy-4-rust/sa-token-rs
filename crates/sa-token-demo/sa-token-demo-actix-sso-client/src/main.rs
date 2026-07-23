//! SSO Client Actix Demo

mod util;

use std::env;
use std::sync::Arc;

use actix_web::{App, HttpRequest, HttpResponse, HttpServer, web};
use sa_token::prelude::{AsyncSaTokenRuntime, AsyncStpUtil, SaTokenConfig, SaTokenDaoMemory};
use sa_token_core::context::sa_token_context_default_impl::SaTokenContextDefaultImpl;
use sa_token_sign::sign::SaSignConfig;
use sa_token_sso::sso::config::SaSsoClientConfig;
use sa_token_sso::sso::strategy::SaSsoClientStrategy;
use sa_token_sso::sso::template::SaSsoClientTemplate;
use serde::Deserialize;
use serde_json::{Value, json};

use crate::util::AjaxJson;

#[derive(Clone)]
struct AppState {
    util: AsyncStpUtil,
    client: Arc<SaSsoClientTemplate>,
    http: reqwest::Client,
    server_url: String,
    self_url: String,
}

#[derive(Debug, Deserialize)]
struct SsoLoginQuery {
    #[serde(default)]
    ticket: String,
    #[serde(default = "default_back")]
    back: String,
}
fn default_back() -> String {
    "/".into()
}

fn token_from_req(req: &HttpRequest) -> Option<String> {
    req.headers()
        .get("satoken")?
        .to_str()
        .ok()
        .map(str::to_string)
}

/// SSO 登录。
async fn sso_login(state: web::Data<AppState>, q: web::Query<SsoLoginQuery>) -> HttpResponse {
    if q.ticket.is_empty() {
        let callback = format!("{}/sso/login", state.self_url);
        return match state.client.build_server_auth_url(&callback, Some(&q.back)) {
            Ok(url) => HttpResponse::Found()
                .append_header(("Location", url))
                .finish(),
            Err(e) => HttpResponse::Ok().json(AjaxJson::error(e.to_string())),
        };
    }
    let url = format!(
        "{}/sso/checkTicket?ticket={}&client=sso-client",
        state.server_url, q.ticket
    );
    match state.http.get(&url).send().await {
        Ok(resp) => {
            let body: Value = resp.json().await.unwrap_or(Value::Null);
            let login_id = body
                .get("data")
                .map(|v| match v {
                    Value::String(s) => s.clone(),
                    other => other.to_string().trim_matches('"').to_string(),
                })
                .unwrap_or_default();
            if login_id.is_empty() || body.get("code").and_then(Value::as_i64) != Some(200) {
                return HttpResponse::Ok()
                    .json(AjaxJson::error(format!("checkTicket 失败: {body}")));
            }
            match state.util.login(&login_id).await {
                Ok(token) => {
                    HttpResponse::Ok().json(AjaxJson::ok_msg("SSO 登录成功").set_data(json!({
                        "token": token, "login_id": login_id, "back": q.back,
                    })))
                }
                Err(e) => HttpResponse::Ok().json(AjaxJson::error(e.to_string())),
            }
        }
        Err(e) => HttpResponse::Ok().json(AjaxJson::error(e.to_string())),
    }
}

/// 注销。
async fn sso_logout(state: web::Data<AppState>, req: HttpRequest) -> HttpResponse {
    if let Some(token) = token_from_req(&req)
        && let Ok(Some(id)) = state.util.get_login_id_by_token(&token).await
    {
        let _ = state.util.logout_by_login_id(&id).await;
    }
    HttpResponse::Ok().json(AjaxJson::ok_msg("已注销"))
}

/// 是否登录。
async fn is_login(state: web::Data<AppState>, req: HttpRequest) -> HttpResponse {
    let logged = match token_from_req(&req) {
        Some(t) => state
            .util
            .get_login_id_by_token(&t)
            .await
            .ok()
            .flatten()
            .is_some(),
        None => false,
    };
    HttpResponse::Ok().json(AjaxJson::ok_data(logged))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let server_url = env::var("SSO_SERVER_URL").unwrap_or_else(|_| "http://127.0.0.1:8096".into());
    let self_url = env::var("SSO_CLIENT_URL").unwrap_or_else(|_| "http://127.0.0.1:8118".into());
    let runtime = AsyncSaTokenRuntime::new(
        Arc::new(SaTokenConfig {
            token_name: "satoken".into(),
            ..Default::default()
        }),
        Arc::new(SaTokenDaoMemory::new()),
        Arc::new(SaTokenContextDefaultImpl),
    );
    let util = AsyncStpUtil::new("login", Arc::new(runtime));
    let config = Arc::new(SaSsoClientConfig {
        client: Some("sso-client".into()),
        server_url: Some(server_url.clone()),
        is_check_sign: false,
        ..Default::default()
    });
    let client = Arc::new(
        SaSsoClientTemplate::new(
            config,
            Arc::new(SaSsoClientStrategy::default()),
            Arc::new(SaSignConfig::default()),
            Arc::new(SaTokenDaoMemory::new()),
            "satoken",
            Arc::new(|_, _| Ok(())),
        )
        .expect("sso client"),
    );

    let state = AppState {
        util,
        client,
        http: reqwest::Client::new(),
        server_url,
        self_url,
    };
    let addr = ("0.0.0.0", 8118);
    println!("🚀 SSO Client Actix Demo  http://{}:{}", addr.0, addr.1);
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(state.clone()))
            .route("/sso/login", web::get().to(sso_login))
            .route("/sso/logout", web::get().to(sso_logout))
            .route("/acc/isLogin", web::get().to(is_login))
    })
    .bind(addr)?
    .run()
    .await
}
