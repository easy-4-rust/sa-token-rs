//! Sa-Token-Rs Axum OAuth2 Server Demo
//!
//! 对应 Java：`sa-token-demo-oauth2-server`（Spring Boot → axum）

mod util;

use std::sync::Arc;

use axum::Json;
use axum::extract::{Query, State};
use axum::http::HeaderMap;
use axum::routing::get;
use axum::{Router, response::IntoResponse};
use sa_token::prelude::*;
use sa_token_web_axum::SaTokenLayer;
use sa_token_dao_memory::SaTokenDaoMemory;
use sa_token_oauth2::oauth2::config::SaOAuth2ServerConfig;
use sa_token_oauth2::oauth2::consts::GrantType;
use sa_token_oauth2::oauth2::dao::SaOAuth2Dao;
use sa_token_oauth2::oauth2::data::convert::{
    SaOAuth2DataConverterDefaultImpl, SaOAuth2TokenGenerator,
};
use sa_token_oauth2::oauth2::data::generate::{
    SaOAuth2DataGenerate, SaOAuth2DataGenerateDefaultImpl, SaOAuth2GenerateHooks,
};
use sa_token_oauth2::oauth2::data::loader::SaOAuth2DataLoaderDefaultImpl;
use sa_token_oauth2::oauth2::data::model::loader::SaClientModel;
use sa_token_oauth2::oauth2::data::model::request::RequestAuthModel;
use sa_token_oauth2::oauth2::template::SaOAuth2Template;
use serde::Deserialize;
use serde_json::json;

use crate::util::AjaxJson;

/// UUID Token 生成器。
struct UuidTokenGenerator;

impl SaOAuth2TokenGenerator for UuidTokenGenerator {
    type Error = String;

    fn create_code(
        &self,
        _: &str,
        _: &serde_json::Value,
        _: &[String],
    ) -> Result<String, Self::Error> {
        Ok(format!("code-{}", uuid::Uuid::new_v4()))
    }

    fn create_access_token(
        &self,
        _: &str,
        _: &serde_json::Value,
        _: &[String],
    ) -> Result<String, Self::Error> {
        Ok(format!("access-{}", uuid::Uuid::new_v4()))
    }

    fn create_refresh_token(
        &self,
        _: &str,
        _: &serde_json::Value,
        _: &[String],
    ) -> Result<String, Self::Error> {
        Ok(format!("refresh-{}", uuid::Uuid::new_v4()))
    }

    fn create_client_token(&self, _: &str, _: &[String]) -> Result<String, Self::Error> {
        Ok(format!("client-{}", uuid::Uuid::new_v4()))
    }
}

struct EmptyHooks;
impl SaOAuth2GenerateHooks for EmptyHooks {}

/// 应用状态。
#[derive(Clone)]
struct AppState {
    generator: Arc<dyn SaOAuth2DataGenerate>,
    template: Arc<SaOAuth2Template>,
}

#[derive(Debug, Deserialize)]
struct LoginQuery {
    #[serde(default = "default_name")]
    name: String,
    #[serde(default = "default_pwd")]
    pwd: String,
}

fn default_name() -> String {
    "sa".into()
}
fn default_pwd() -> String {
    "123456".into()
}

#[derive(Debug, Deserialize)]
struct ClientTokenQuery {
    client_id: String,
    client_secret: String,
    #[serde(default = "default_scope")]
    scope: String,
}

fn default_scope() -> String {
    "userinfo".into()
}

#[derive(Debug, Deserialize)]
struct AuthorizeQuery {
    client_id: String,
    #[serde(default = "default_redirect")]
    redirect_uri: String,
    #[serde(default = "default_scope")]
    scope: String,
}

fn default_redirect() -> String {
    "http://localhost:9001/callback".into()
}

#[derive(Debug, Deserialize)]
struct TokenQuery {
    code: String,
}

fn build_state() -> AppState {
    SaManager::set_config(Arc::new(SaTokenConfig::default()));
    SaManager::set_sa_token_dao(Arc::new(SaTokenDaoMemory::new()));
    SaManager::put_stp_logic(Arc::new(StpLogic::new("login")));

    let mut config = SaOAuth2ServerConfig::default();
    let mut client = SaClientModel::from_server_config(&config);
    client.client_id = Some("1001".into());
    client.client_secret = Some("aaaa-bbbb-cccc-dddd-eeee".into());
    client.contract_scopes = vec!["openid".into(), "userinfo".into(), "openid".into()];
    client.allow_redirect_uris = vec!["http://localhost:9001/callback".into(), "*".into()];
    client.allow_grant_types = vec![
        GrantType::AUTHORIZATION_CODE.into(),
        GrantType::REFRESH_TOKEN.into(),
        GrantType::CLIENT_CREDENTIALS.into(),
        GrantType::PASSWORD.into(),
    ];
    client.is_auto_confirm = true;
    config.add_client(client).expect("add client");
    let config = Arc::new(config);

    let dao = Arc::new(SaOAuth2Dao::new(
        Arc::new(SaTokenDaoMemory::new()),
        "satoken",
        300,
    ));
    let converter = Arc::new(SaOAuth2DataConverterDefaultImpl::new(Arc::new(
        UuidTokenGenerator,
    )));
    let loader = Arc::new(SaOAuth2DataLoaderDefaultImpl::new(Arc::clone(&config)));
    let generator: Arc<dyn SaOAuth2DataGenerate> = Arc::new(SaOAuth2DataGenerateDefaultImpl::new(
        Arc::clone(&dao),
        Arc::clone(&converter),
        Arc::clone(&loader),
        Arc::new(EmptyHooks),
    ));
    let template = Arc::new(SaOAuth2Template::new(loader, dao));

    AppState {
        generator,
        template,
    }
}

/// OAuth2 登录 —— `/oauth2/doLogin`
async fn do_login(Query(q): Query<LoginQuery>) -> impl IntoResponse {
    if q.name == "sa" && q.pwd == "123456" {
        match StpUtil::login("10001") {
            Ok(()) => {
                let token = StpUtil::get_token_value().unwrap_or_default();
                Json(AjaxJson::ok().set("satoken", token))
            }
            Err(e) => Json(AjaxJson::error(e.to_string())),
        }
    } else {
        Json(AjaxJson::error("账号名或密码错误"))
    }
}

/// client_credentials —— `/oauth2/client_token`
async fn client_token(
    State(state): State<AppState>,
    Query(q): Query<ClientTokenQuery>,
) -> impl IntoResponse {
    if let Err(e) = state
        .template
        .check_client_secret(&q.client_id, &q.client_secret)
    {
        return Json(AjaxJson::error(e.to_string()));
    }
    let scopes: Vec<String> = q
        .scope
        .split([',', ' '])
        .filter(|s| !s.is_empty())
        .map(str::to_string)
        .collect();
    match state
        .generator
        .generate_client_token(&q.client_id, &scopes)
        .await
    {
        Ok(token) => Json(AjaxJson::ok_data(token)),
        Err(e) => Json(AjaxJson::error(e.to_string())),
    }
}

/// 授权码 —— `/oauth2/authorize`（需登录）
async fn authorize(
    State(state): State<AppState>,
    Query(q): Query<AuthorizeQuery>,
) -> impl IntoResponse {
    let Ok(login_id) = StpUtil::get_login_id() else {
        return Json(AjaxJson::error("未登录，请先调用 /oauth2/doLogin"));
    };
    let scopes: Vec<String> = q
        .scope
        .split([',', ' '])
        .filter(|s| !s.is_empty())
        .map(str::to_string)
        .collect();
    let request = RequestAuthModel {
        client_id: Some(q.client_id),
        scopes: Some(scopes),
        login_id: Some(json!(login_id)),
        redirect_uri: Some(q.redirect_uri),
        ..Default::default()
    };
    match state.generator.generate_code(&request).await {
        Ok(code) => Json(AjaxJson::ok_data(code)),
        Err(e) => Json(AjaxJson::error(e.to_string())),
    }
}

/// code 换 token —— `/oauth2/token`
async fn token(State(state): State<AppState>, Query(q): Query<TokenQuery>) -> impl IntoResponse {
    match state.generator.generate_access_token(&q.code).await {
        Ok(access) => Json(AjaxJson::ok_data(access)),
        Err(e) => Json(AjaxJson::error(e.to_string())),
    }
}

/// 资源接口 —— `/oauth2/userinfo`
async fn userinfo(State(state): State<AppState>, headers: HeaderMap) -> impl IntoResponse {
    let access_token = headers
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .or_else(|| headers.get("access_token").and_then(|v| v.to_str().ok()));
    let Some(token) = access_token else {
        return Json(AjaxJson::error("缺少 access_token"));
    };
    match state.template.check_access_token(token).await {
        Ok(model) => Json(AjaxJson::ok_data(json!({
            "login_id": model.login_id,
            "scopes": model.scopes,
            "client_id": model.client_id,
        }))),
        Err(e) => Json(AjaxJson::error(e.to_string())),
    }
}

#[tokio::main]
async fn main() {
    let state = build_state();
    let app = Router::new()
        .route("/oauth2/doLogin", get(do_login).post(do_login))
        .route("/oauth2/client_token", get(client_token).post(client_token))
        .route("/oauth2/authorize", get(authorize))
        .route("/oauth2/token", get(token).post(token))
        .route("/oauth2/userinfo", get(userinfo))
        .with_state(state)
        .layer(SaTokenLayer::new());

    let addr = "0.0.0.0:8093";
    println!("🚀 Sa-Token-Rs Axum OAuth2 Demo");
    println!("   http://{addr}");
    println!("   client_id=1001  secret=aaaa-bbbb-cccc-dddd-eeee");

    let listener = tokio::net::TcpListener::bind(addr).await.expect("bind");
    axum::serve(listener, app).await.expect("serve");
}
