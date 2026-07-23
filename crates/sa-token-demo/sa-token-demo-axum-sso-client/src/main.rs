//! SSO Client Demo（Mode3）：跳转 auth → ticket → checkTicket → 本地登录

mod util;

use std::env;
use std::sync::Arc;

use axum::Json;
use axum::Router;
use axum::extract::{Query, State};
use axum::http::HeaderMap;
use axum::response::{IntoResponse, Redirect, Response};
use axum::routing::get;
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

fn token_from_headers(headers: &HeaderMap) -> Option<String> {
    headers
        .get("satoken")
        .and_then(|v| v.to_str().ok())
        .map(str::to_string)
}

/// SSO 登录入口。
async fn sso_login(State(state): State<AppState>, Query(q): Query<SsoLoginQuery>) -> Response {
    if q.ticket.is_empty() {
        let callback = format!("{}/sso/login", state.self_url);
        match state.client.build_server_auth_url(&callback, Some(&q.back)) {
            Ok(url) => Redirect::temporary(&url).into_response(),
            Err(e) => Json(AjaxJson::error(e.to_string())).into_response(),
        }
    } else {
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
                    return Json(AjaxJson::error(format!("checkTicket 失败: {body}")))
                        .into_response();
                }
                match state.util.login(&login_id).await {
                    Ok(token) => Json(AjaxJson::ok_msg("SSO 登录成功").set_data(json!({
                        "token": token,
                        "login_id": login_id,
                        "back": q.back,
                    })))
                    .into_response(),
                    Err(e) => Json(AjaxJson::error(e.to_string())).into_response(),
                }
            }
            Err(e) => Json(AjaxJson::error(e.to_string())).into_response(),
        }
    }
}

/// 注销。
async fn sso_logout(State(state): State<AppState>, headers: HeaderMap) -> Json<AjaxJson> {
    if let Some(token) = token_from_headers(&headers)
        && let Ok(Some(id)) = state.util.get_login_id_by_token(&token).await
    {
        let _ = state.util.logout_by_login_id(&id).await;
    }
    Json(AjaxJson::ok_msg("已注销"))
}

/// 是否登录。
async fn is_login(State(state): State<AppState>, headers: HeaderMap) -> Json<AjaxJson> {
    let logged = match token_from_headers(&headers) {
        Some(t) => state
            .util
            .get_login_id_by_token(&t)
            .await
            .ok()
            .flatten()
            .is_some(),
        None => false,
    };
    Json(AjaxJson::ok_data(logged))
}

#[tokio::main]
async fn main() {
    let server_url = env::var("SSO_SERVER_URL").unwrap_or_else(|_| "http://127.0.0.1:8095".into());
    let self_url = env::var("SSO_CLIENT_URL").unwrap_or_else(|_| "http://127.0.0.1:8117".into());

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
    let strategy = Arc::new(SaSsoClientStrategy::default());
    let dao = Arc::new(SaTokenDaoMemory::new());
    let client = Arc::new(
        SaSsoClientTemplate::new(
            config,
            strategy,
            Arc::new(SaSignConfig::default()),
            dao,
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

    let app = Router::new()
        .route("/sso/login", get(sso_login))
        .route("/sso/logout", get(sso_logout))
        .route("/acc/isLogin", get(is_login))
        .with_state(state);

    let addr = "0.0.0.0:8117";
    println!("🚀 SSO Client Axum Demo  http://{addr}");
    println!("   先启动 sa-token-demo-axum-sso (:8095)");
    let listener = tokio::net::TcpListener::bind(addr).await.expect("bind");
    axum::serve(listener, app).await.expect("serve");
}
