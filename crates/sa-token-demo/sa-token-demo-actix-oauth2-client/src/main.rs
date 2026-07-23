//! OAuth2 Client Actix Demo

mod util;

use std::env;

use actix_web::{App, HttpResponse, HttpServer, web};
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

/// 帮助。
async fn index(state: web::Data<AppState>) -> HttpResponse {
    HttpResponse::Ok().json(AjaxJson::ok_data(json!({
        "server": state.server,
        "endpoints": ["/oauth2/codeLogin", "/oauth2/passwordLogin", "/oauth2/clientToken"],
    })))
}

/// codeLogin。
async fn code_login(state: web::Data<AppState>, q: web::Query<CodeLoginQuery>) -> HttpResponse {
    let url = format!("{}/oauth2/token?code={}", state.server, q.code);
    match state.http.get(&url).send().await {
        Ok(resp) => {
            HttpResponse::Ok().json(AjaxJson::ok_data(resp.json().await.unwrap_or(Value::Null)))
        }
        Err(e) => HttpResponse::Ok().json(AjaxJson::error(e.to_string())),
    }
}

/// passwordLogin 组合调用。
async fn password_login(
    state: web::Data<AppState>,
    q: web::Query<PasswordLoginQuery>,
) -> HttpResponse {
    let login_url = format!(
        "{}/oauth2/doLogin?name={}&pwd={}",
        state.server, q.username, q.password
    );
    let Ok(login_resp) = state.http.get(&login_url).send().await else {
        return HttpResponse::Ok().json(AjaxJson::error("login request failed"));
    };
    let login_json: Value = login_resp.json().await.unwrap_or(Value::Null);
    let satoken = login_json
        .pointer("/data/satoken")
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string();
    if satoken.is_empty() {
        return HttpResponse::Ok().json(AjaxJson::error(format!("server 登录失败: {login_json}")));
    }
    let auth_url = format!(
        "{}/oauth2/authorize?client_id=1001&redirect_uri=http://localhost:9001/callback&scope=userinfo",
        state.server
    );
    let Ok(auth_resp) = state
        .http
        .get(&auth_url)
        .header("satoken", &satoken)
        .send()
        .await
    else {
        return HttpResponse::Ok().json(AjaxJson::error("authorize failed"));
    };
    let auth_json: Value = auth_resp.json().await.unwrap_or(Value::Null);
    let code = auth_json
        .pointer("/data/code")
        .and_then(Value::as_str)
        .or_else(|| auth_json.pointer("/data").and_then(Value::as_str))
        .unwrap_or("")
        .to_string();
    if code.is_empty() {
        return HttpResponse::Ok().json(AjaxJson::error(format!("authorize 失败: {auth_json}")));
    }
    let token_url = format!("{}/oauth2/token?code={code}", state.server);
    match state.http.get(&token_url).send().await {
        Ok(resp) => {
            HttpResponse::Ok().json(AjaxJson::ok_data(resp.json().await.unwrap_or(Value::Null)))
        }
        Err(e) => HttpResponse::Ok().json(AjaxJson::error(e.to_string())),
    }
}

/// refresh 说明。
async fn refresh(state: web::Data<AppState>, q: web::Query<RefreshQuery>) -> HttpResponse {
    HttpResponse::Ok().json(AjaxJson::ok_data(json!({
        "note": "demo server 未单独暴露 refresh HTTP",
        "refresh_token": q.refresh_token,
        "server": state.server,
    })))
}

/// clientToken。
async fn client_token(state: web::Data<AppState>) -> HttpResponse {
    let url = format!(
        "{}/oauth2/client_token?client_id=1001&client_secret=aaaa-bbbb-cccc-dddd-eeee&scope=userinfo",
        state.server
    );
    match state.http.get(&url).send().await {
        Ok(resp) => {
            HttpResponse::Ok().json(AjaxJson::ok_data(resp.json().await.unwrap_or(Value::Null)))
        }
        Err(e) => HttpResponse::Ok().json(AjaxJson::error(e.to_string())),
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let server = env::var("OAUTH2_SERVER_URL").unwrap_or_else(|_| "http://127.0.0.1:8094".into());
    let state = AppState {
        server,
        http: reqwest::Client::new(),
    };
    let addr = ("0.0.0.0", 8116);
    println!("🚀 OAuth2 Client Actix Demo  http://{}:{}", addr.0, addr.1);
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(state.clone()))
            .route("/", web::get().to(index))
            .route("/oauth2/codeLogin", web::get().to(code_login))
            .route("/oauth2/passwordLogin", web::get().to(password_login))
            .route("/oauth2/refresh", web::get().to(refresh))
            .route("/oauth2/clientToken", web::get().to(client_token))
    })
    .bind(addr)?
    .run()
    .await
}
