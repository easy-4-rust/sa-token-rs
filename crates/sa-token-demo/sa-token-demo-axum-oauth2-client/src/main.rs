//! OAuth2 Client Demo：通过 reqwest 调用本地 oauth2-server

mod util;

use std::env;

use axum::Json;
use axum::Router;
use axum::extract::{Query, State};
use axum::routing::get;
use serde::Deserialize;
use serde_json::{Value, json};

use crate::util::AjaxJson;

#[derive(Clone)]
struct AppState {
    server: String,
    http: reqwest::Client,
}

#[derive(Debug, Deserialize)]
struct CodeLoginQuery {
    code: String,
}

#[derive(Debug, Deserialize)]
struct PasswordLoginQuery {
    #[serde(default = "default_user")]
    username: String,
    #[serde(default = "default_pwd")]
    password: String,
}
fn default_user() -> String {
    "sa".into()
}
fn default_pwd() -> String {
    "123456".into()
}

#[derive(Debug, Deserialize)]
struct RefreshQuery {
    refresh_token: String,
}

/// 帮助页。
async fn index(State(state): State<AppState>) -> Json<AjaxJson> {
    Json(AjaxJson::ok_data(json!({
        "server": state.server,
        "endpoints": [
            "/oauth2/codeLogin?code=",
            "/oauth2/passwordLogin?username=sa&password=123456",
            "/oauth2/refresh?refresh_token=",
            "/oauth2/clientToken",
        ],
        "note": "先启动 sa-token-demo-axum-oauth2 (:8093)",
    })))
}

/// code 换 token。
async fn code_login(
    State(state): State<AppState>,
    Query(q): Query<CodeLoginQuery>,
) -> Json<AjaxJson> {
    let url = format!("{}/oauth2/token?code={}", state.server, q.code);
    match state.http.get(&url).send().await {
        Ok(resp) => {
            let body: Value = resp.json().await.unwrap_or(Value::Null);
            Json(AjaxJson::ok_data(body))
        }
        Err(e) => Json(AjaxJson::error(e.to_string())),
    }
}

/// 密码模式：先 server 登录拿 satoken，再 authorize 拿 code，再换 token。
async fn password_login(
    State(state): State<AppState>,
    Query(q): Query<PasswordLoginQuery>,
) -> Json<AjaxJson> {
    let login_url = format!(
        "{}/oauth2/doLogin?name={}&pwd={}",
        state.server, q.username, q.password
    );
    let login_resp = match state.http.get(&login_url).send().await {
        Ok(r) => r,
        Err(e) => return Json(AjaxJson::error(e.to_string())),
    };
    let login_json: Value = login_resp.json().await.unwrap_or(Value::Null);
    let satoken = login_json
        .pointer("/data/satoken")
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string();
    if satoken.is_empty() {
        return Json(AjaxJson::error(format!("server 登录失败: {login_json}")));
    }
    let auth_url = format!(
        "{}/oauth2/authorize?client_id=1001&redirect_uri=http://localhost:9001/callback&scope=userinfo",
        state.server
    );
    let auth_resp = match state
        .http
        .get(&auth_url)
        .header("satoken", &satoken)
        .send()
        .await
    {
        Ok(r) => r,
        Err(e) => return Json(AjaxJson::error(e.to_string())),
    };
    let auth_json: Value = auth_resp.json().await.unwrap_or(Value::Null);
    let code = auth_json
        .pointer("/data/code")
        .or_else(|| auth_json.pointer("/data"))
        .cloned()
        .unwrap_or(Value::Null);
    let code_str = match code {
        Value::String(s) => s,
        Value::Object(o) => o
            .get("code")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string(),
        other => other.as_str().unwrap_or("").to_string(),
    };
    if code_str.is_empty() {
        return Json(AjaxJson::error(format!("authorize 失败: {auth_json}")));
    }
    let token_url = format!("{}/oauth2/token?code={code_str}", state.server);
    match state.http.get(&token_url).send().await {
        Ok(resp) => Json(AjaxJson::ok_data(resp.json().await.unwrap_or(Value::Null))),
        Err(e) => Json(AjaxJson::error(e.to_string())),
    }
}

/// refresh（demo server 若未暴露则返回说明）。
async fn refresh(State(state): State<AppState>, Query(q): Query<RefreshQuery>) -> Json<AjaxJson> {
    Json(AjaxJson::ok_data(json!({
        "note": "当前 axum-oauth2 demo 未单独暴露 refresh HTTP；请在完整 OAuth2 Server 上调用 refresh_token grant",
        "refresh_token": q.refresh_token,
        "server": state.server,
    })))
}

/// client_credentials。
async fn client_token(State(state): State<AppState>) -> Json<AjaxJson> {
    let url = format!(
        "{}/oauth2/client_token?client_id=1001&client_secret=aaaa-bbbb-cccc-dddd-eeee&scope=userinfo",
        state.server
    );
    match state.http.get(&url).send().await {
        Ok(resp) => Json(AjaxJson::ok_data(resp.json().await.unwrap_or(Value::Null))),
        Err(e) => Json(AjaxJson::error(e.to_string())),
    }
}

#[tokio::main]
async fn main() {
    let server = env::var("OAUTH2_SERVER_URL").unwrap_or_else(|_| "http://127.0.0.1:8093".into());
    let state = AppState {
        server,
        http: reqwest::Client::new(),
    };
    let app = Router::new()
        .route("/", get(index))
        .route("/oauth2/codeLogin", get(code_login))
        .route("/oauth2/passwordLogin", get(password_login))
        .route("/oauth2/refresh", get(refresh))
        .route("/oauth2/clientToken", get(client_token))
        .with_state(state);
    let addr = "0.0.0.0:8115";
    println!("🚀 OAuth2 Client Axum Demo  http://{addr}");
    let listener = tokio::net::TcpListener::bind(addr).await.expect("bind");
    axum::serve(listener, app).await.expect("serve");
}
