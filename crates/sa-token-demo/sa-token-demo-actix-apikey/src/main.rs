//! Sa-Token-Rs Actix ApiKey Demo（Quarkus → actix）

mod util;

use std::sync::Arc;

use actix_web::{App, HttpRequest, HttpResponse, HttpServer, web};
use sa_token::prelude::{AsyncSaTokenRuntime, AsyncStpUtil, SaTokenConfig, SaTokenDaoMemory};
use sa_token_web_actix::{RequireLogin, require_login};
use sa_token_apikey::apikey::loader::SaApiKeyDataLoaderDefaultImpl;
use sa_token_apikey::{SaApiKeyConfig, SaApiKeyManager};
use sa_token_core::context::sa_token_context_default_impl::SaTokenContextDefaultImpl;
use serde::Deserialize;
use serde_json::json;

use crate::util::AjaxJson;

/// 应用状态。
#[derive(Clone)]
struct AppState {
    util: AsyncStpUtil,
    manager: Arc<SaApiKeyManager>,
}

#[derive(Debug, Deserialize)]
struct LoginQuery {
    #[serde(default = "default_id")]
    id: String,
}

fn default_id() -> String {
    "10001".into()
}

#[derive(Debug, Deserialize)]
struct DeleteQuery {
    api_key: String,
}

fn build_state() -> AppState {
    let dao = Arc::new(SaTokenDaoMemory::new());
    let runtime = AsyncSaTokenRuntime::new(
        Arc::new(SaTokenConfig::default()),
        Arc::clone(&dao) as Arc<_>,
        Arc::new(SaTokenContextDefaultImpl),
    );
    let manager = SaApiKeyManager::new(
        "satoken",
        Arc::new(SaApiKeyConfig::default()),
        dao,
        Arc::new(SaApiKeyDataLoaderDefaultImpl),
    )
    .expect("apikey manager");
    AppState {
        util: AsyncStpUtil::new("login", Arc::new(runtime)),
        manager: Arc::new(manager),
    }
}

fn read_api_key(req: &HttpRequest) -> Option<String> {
    req.headers()
        .get("apikey")
        .and_then(|v| v.to_str().ok())
        .map(str::to_string)
        .or_else(|| {
            web::Query::<std::collections::HashMap<String, String>>::from_query(req.query_string())
                .ok()
                .and_then(|q| q.get("apikey").cloned())
        })
}

async fn login(state: web::Data<AppState>, query: web::Query<LoginQuery>) -> HttpResponse {
    match state.util.login(&query.id).await {
        Ok(token) => HttpResponse::Ok().json(AjaxJson::ok().set_data(json!({ "satoken": token }))),
        Err(e) => HttpResponse::Ok().json(AjaxJson::error(e.to_string())),
    }
}

async fn create_api_key(state: web::Data<AppState>, login: RequireLogin) -> HttpResponse {
    let template = state.manager.template();
    match template.create_api_key_model(&login.0.login_id).await {
        Ok(mut model) => {
            model.title = Some("test".into());
            model.scopes = vec!["userinfo".into(), "chat".into()];
            if let Err(e) = template.save_api_key(&model).await {
                return HttpResponse::Ok().json(AjaxJson::error(e.to_string()));
            }
            HttpResponse::Ok().json(AjaxJson::ok_data(model))
        }
        Err(e) => HttpResponse::Ok().json(AjaxJson::error(e.to_string())),
    }
}

async fn my_api_key_list(state: web::Data<AppState>, login: RequireLogin) -> HttpResponse {
    match state
        .manager
        .template()
        .get_api_key_list(&login.0.login_id)
        .await
    {
        Ok(list) => HttpResponse::Ok().json(AjaxJson::ok_data(list)),
        Err(e) => HttpResponse::Ok().json(AjaxJson::error(e.to_string())),
    }
}

async fn delete_api_key(
    state: web::Data<AppState>,
    login: RequireLogin,
    query: web::Query<DeleteQuery>,
) -> HttpResponse {
    let template = state.manager.template();
    if let Err(e) = template
        .check_api_key_login_id(&query.api_key, &login.0.login_id)
        .await
    {
        return HttpResponse::Ok().json(AjaxJson::error(e.to_string()));
    }
    match template.delete_api_key(&query.api_key).await {
        Ok(()) => HttpResponse::Ok().json(AjaxJson::ok()),
        Err(e) => HttpResponse::Ok().json(AjaxJson::error(e.to_string())),
    }
}

async fn ak_res1(state: web::Data<AppState>, req: HttpRequest) -> HttpResponse {
    let Some(api_key) = read_api_key(&req) else {
        return HttpResponse::Ok().json(AjaxJson::error("缺少 apikey"));
    };
    match state.manager.template().check_api_key(&api_key).await {
        Ok(model) => HttpResponse::Ok().json(AjaxJson::ok_msg("调用成功").set_data(model)),
        Err(e) => HttpResponse::Ok().json(AjaxJson::error(e.to_string())),
    }
}

async fn ak_res2(state: web::Data<AppState>, req: HttpRequest) -> HttpResponse {
    let Some(api_key) = read_api_key(&req) else {
        return HttpResponse::Ok().json(AjaxJson::error("缺少 apikey"));
    };
    match state
        .manager
        .template()
        .check_api_key_scope(&api_key, &["userinfo"])
        .await
    {
        Ok(model) => HttpResponse::Ok().json(AjaxJson::ok_msg("调用成功").set_data(model)),
        Err(e) => HttpResponse::Ok().json(AjaxJson::error(e.to_string())),
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let state = build_state();
    let addr = ("0.0.0.0", 8092);
    println!("🚀 Sa-Token-Rs Actix ApiKey Demo");
    println!("   http://{}:{}", addr.0, addr.1);

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(state.clone()))
            .app_data(web::Data::new(state.util.clone()))
            .route("/login", web::get().to(login))
            .route("/akRes1", web::get().to(ak_res1))
            .route("/akRes2", web::get().to(ak_res2))
            .service(
                web::scope("/secure")
                    .wrap(actix_web::middleware::from_fn(require_login))
                    .route("/createApiKey", web::get().to(create_api_key))
                    .route("/myApiKeyList", web::get().to(my_api_key_list))
                    .route("/deleteApiKey", web::get().to(delete_api_key)),
            )
    })
    .bind(addr)?
    .run()
    .await
}
