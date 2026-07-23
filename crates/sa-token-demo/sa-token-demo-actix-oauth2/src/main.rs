//! Sa-Token-Rs Actix OAuth2 Server Demo
//!
//! 对应 Java：`sa-token-demo-oauth2-server`（Spring Boot → actix-web）
//! 镜像：`sa-token-demo-axum-oauth2`，使用 `AsyncStpUtil`。

mod util;

use std::sync::Arc;

use actix_web::{App, HttpRequest, HttpResponse, HttpServer, middleware, web};
use sa_token::prelude::{AsyncSaTokenRuntime, AsyncStpUtil, SaTokenConfig};
use sa_token_web_actix::{RequireLogin, require_login};
use sa_token_core::context::sa_token_context_default_impl::SaTokenContextDefaultImpl;
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
    util: AsyncStpUtil,
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

/// 初始化 OAuth2 与 AsyncStpUtil。
fn build_state() -> AppState {
    let dao = Arc::new(SaTokenDaoMemory::new());
    let runtime = AsyncSaTokenRuntime::new(
        Arc::new(SaTokenConfig::default()),
        Arc::clone(&dao) as Arc<_>,
        Arc::new(SaTokenContextDefaultImpl),
    );
    let util = AsyncStpUtil::new("login", Arc::new(runtime));

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

    let oauth_dao = Arc::new(SaOAuth2Dao::new(
        Arc::new(SaTokenDaoMemory::new()),
        "satoken",
        300,
    ));
    let converter = Arc::new(SaOAuth2DataConverterDefaultImpl::new(Arc::new(
        UuidTokenGenerator,
    )));
    let loader = Arc::new(SaOAuth2DataLoaderDefaultImpl::new(Arc::clone(&config)));
    let generator: Arc<dyn SaOAuth2DataGenerate> = Arc::new(SaOAuth2DataGenerateDefaultImpl::new(
        Arc::clone(&oauth_dao),
        Arc::clone(&converter),
        Arc::clone(&loader),
        Arc::new(EmptyHooks),
    ));
    let template = Arc::new(SaOAuth2Template::new(loader, oauth_dao));

    AppState {
        util,
        generator,
        template,
    }
}

/// OAuth2 登录 —— `/oauth2/doLogin`
async fn do_login(state: web::Data<AppState>, query: web::Query<LoginQuery>) -> HttpResponse {
    if query.name == "sa" && query.pwd == "123456" {
        match state.util.login("10001").await {
            Ok(token) => HttpResponse::Ok().json(AjaxJson::ok().set("satoken", token)),
            Err(e) => HttpResponse::Ok().json(AjaxJson::error(e.to_string())),
        }
    } else {
        HttpResponse::Ok().json(AjaxJson::error("账号名或密码错误"))
    }
}

/// client_credentials —— `/oauth2/client_token`
async fn client_token(
    state: web::Data<AppState>,
    query: web::Query<ClientTokenQuery>,
) -> HttpResponse {
    if let Err(e) = state
        .template
        .check_client_secret(&query.client_id, &query.client_secret)
    {
        return HttpResponse::Ok().json(AjaxJson::error(e.to_string()));
    }
    let scopes: Vec<String> = query
        .scope
        .split([',', ' '])
        .filter(|s| !s.is_empty())
        .map(str::to_string)
        .collect();
    match state
        .generator
        .generate_client_token(&query.client_id, &scopes)
        .await
    {
        Ok(token) => HttpResponse::Ok().json(AjaxJson::ok_data(token)),
        Err(e) => HttpResponse::Ok().json(AjaxJson::error(e.to_string())),
    }
}

/// 授权码 —— `/oauth2/authorize`（需登录）
async fn authorize(
    state: web::Data<AppState>,
    login: RequireLogin,
    query: web::Query<AuthorizeQuery>,
) -> HttpResponse {
    let scopes: Vec<String> = query
        .scope
        .split([',', ' '])
        .filter(|s| !s.is_empty())
        .map(str::to_string)
        .collect();
    let request = RequestAuthModel {
        client_id: Some(query.client_id.clone()),
        scopes: Some(scopes),
        login_id: Some(json!(login.0.login_id)),
        redirect_uri: Some(query.redirect_uri.clone()),
        ..Default::default()
    };
    match state.generator.generate_code(&request).await {
        Ok(code) => HttpResponse::Ok().json(AjaxJson::ok_data(code)),
        Err(e) => HttpResponse::Ok().json(AjaxJson::error(e.to_string())),
    }
}

/// code 换 token —— `/oauth2/token`
async fn token(state: web::Data<AppState>, query: web::Query<TokenQuery>) -> HttpResponse {
    match state.generator.generate_access_token(&query.code).await {
        Ok(access) => HttpResponse::Ok().json(AjaxJson::ok_data(access)),
        Err(e) => HttpResponse::Ok().json(AjaxJson::error(e.to_string())),
    }
}

/// 资源接口 —— `/oauth2/userinfo`
async fn userinfo(state: web::Data<AppState>, req: HttpRequest) -> HttpResponse {
    let access_token = req
        .headers()
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .or_else(|| {
            req.headers()
                .get("access_token")
                .and_then(|v| v.to_str().ok())
        });
    let Some(token) = access_token else {
        return HttpResponse::Ok().json(AjaxJson::error("缺少 access_token"));
    };
    match state.template.check_access_token(token).await {
        Ok(model) => HttpResponse::Ok().json(AjaxJson::ok_data(json!({
            "login_id": model.login_id,
            "scopes": model.scopes,
            "client_id": model.client_id,
        }))),
        Err(e) => HttpResponse::Ok().json(AjaxJson::error(e.to_string())),
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let state = build_state();
    let addr = ("0.0.0.0", 8094);
    println!("🚀 Sa-Token-Rs Actix OAuth2 Demo");
    println!("   http://{}:{}", addr.0, addr.1);
    println!("   client_id=1001  secret=aaaa-bbbb-cccc-dddd-eeee");

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(state.clone()))
            .app_data(web::Data::new(state.util.clone()))
            .route("/oauth2/doLogin", web::get().to(do_login))
            .route("/oauth2/doLogin", web::post().to(do_login))
            .route("/oauth2/client_token", web::get().to(client_token))
            .route("/oauth2/client_token", web::post().to(client_token))
            .route("/oauth2/token", web::get().to(token))
            .route("/oauth2/token", web::post().to(token))
            .route("/oauth2/userinfo", web::get().to(userinfo))
            .service(
                web::scope("/oauth2")
                    .wrap(middleware::from_fn(require_login))
                    .route("/authorize", web::get().to(authorize)),
            )
    })
    .bind(addr)?
    .run()
    .await
}
