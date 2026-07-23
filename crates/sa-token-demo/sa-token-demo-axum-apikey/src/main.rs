//! Sa-Token-Rs Axum ApiKey Demo
//!
//! 对应 Java：`sa-token-demo-apikey`（Spring Boot → axum，Jackson → serde）

mod util;

use std::sync::Arc;

use axum::Json;
use axum::extract::{Query, State};
use axum::http::HeaderMap;
use axum::routing::{get, post};
use axum::{Router, response::IntoResponse};
use sa_token::prelude::*;
use sa_token_apikey::apikey::loader::SaApiKeyDataLoaderDefaultImpl;
use sa_token_apikey::{SaApiKeyConfig, SaApiKeyManager};
use sa_token_web_axum::SaTokenLayer;
use serde::Deserialize;
use serde_json::json;

use crate::util::AjaxJson;

/// 应用状态：ApiKey 模板。
#[derive(Clone)]
struct AppState {
    /// ApiKey 管理器
    manager: Arc<SaApiKeyManager>,
}

/// 登录参数。
#[derive(Debug, Deserialize)]
struct LoginQuery {
    #[serde(default = "default_id")]
    id: String,
}

fn default_id() -> String {
    "10001".into()
}

/// 更新 ApiKey 参数。
#[derive(Debug, Deserialize)]
struct UpdateApiKeyBody {
    api_key: String,
    #[serde(default)]
    title: Option<String>,
    #[serde(default)]
    is_valid: Option<bool>,
    #[serde(default)]
    scopes: Option<Vec<String>>,
}

/// 删除参数。
#[derive(Debug, Deserialize)]
struct DeleteQuery {
    api_key: String,
}

/// 初始化。
fn init() -> AppState {
    SaManager::set_config(Arc::new(SaTokenConfig::default()));
    SaManager::set_sa_token_dao(Arc::new(SaTokenDaoMemory::new()));
    SaManager::put_stp_logic(Arc::new(StpLogic::new("login")));

    let dao = Arc::new(SaTokenDaoMemory::new());
    let manager = SaApiKeyManager::new(
        "satoken",
        Arc::new(SaApiKeyConfig::default()),
        dao,
        Arc::new(SaApiKeyDataLoaderDefaultImpl),
    )
    .expect("apikey manager");

    AppState {
        manager: Arc::new(manager),
    }
}

/// 从 Header / Query 读取 apikey。
fn read_api_key(headers: &HeaderMap, query: Option<&str>) -> Option<String> {
    query
        .filter(|v| !v.is_empty())
        .map(str::to_string)
        .or_else(|| {
            headers
                .get("apikey")
                .and_then(|v| v.to_str().ok())
                .map(str::to_string)
        })
}

/// `/login`
async fn login(Query(q): Query<LoginQuery>) -> impl IntoResponse {
    match StpUtil::login(&q.id) {
        Ok(()) => {
            let token = StpUtil::get_token_value().unwrap_or_default();
            Json(AjaxJson::ok().set_data(json!({ "satoken": token })))
        }
        Err(e) => Json(AjaxJson::error(e.to_string())),
    }
}

/// `/getLoginId`
async fn get_login_id() -> impl IntoResponse {
    match StpUtil::get_login_id() {
        Ok(id) => Json(AjaxJson::ok_data(id)),
        Err(e) => Json(AjaxJson::error(e.to_string())),
    }
}

/// `/logout`
async fn logout() -> impl IntoResponse {
    let _ = StpUtil::logout();
    Json(AjaxJson::ok())
}

/// `/myApiKeyList`
async fn my_api_key_list(State(state): State<AppState>) -> impl IntoResponse {
    let Ok(login_id) = StpUtil::get_login_id() else {
        return Json(AjaxJson::error("未登录"));
    };
    match state.manager.template().get_api_key_list(&login_id).await {
        Ok(list) => Json(AjaxJson::ok_data(list)),
        Err(e) => Json(AjaxJson::error(e.to_string())),
    }
}

/// `/createApiKey`
async fn create_api_key(State(state): State<AppState>) -> impl IntoResponse {
    let Ok(login_id) = StpUtil::get_login_id() else {
        return Json(AjaxJson::error("未登录"));
    };
    let template = state.manager.template();
    match template.create_api_key_model(&login_id).await {
        Ok(mut model) => {
            model.title = Some("test".into());
            model.scopes = vec!["userinfo".into(), "chat".into()];
            if let Err(e) = template.save_api_key(&model).await {
                return Json(AjaxJson::error(e.to_string()));
            }
            Json(AjaxJson::ok_data(model))
        }
        Err(e) => Json(AjaxJson::error(e.to_string())),
    }
}

/// `/updateApiKey`
async fn update_api_key(
    State(state): State<AppState>,
    Json(body): Json<UpdateApiKeyBody>,
) -> impl IntoResponse {
    let Ok(login_id) = StpUtil::get_login_id() else {
        return Json(AjaxJson::error("未登录"));
    };
    let template = state.manager.template();
    if let Err(e) = template
        .check_api_key_login_id(&body.api_key, &login_id)
        .await
    {
        return Json(AjaxJson::error(e.to_string()));
    }
    match template.get_api_key(&body.api_key).await {
        Ok(Some(mut model)) => {
            if let Some(title) = body.title {
                model.title = Some(title);
            }
            if let Some(valid) = body.is_valid {
                model.is_valid = valid;
            }
            if let Some(scopes) = body.scopes {
                model.scopes = scopes;
            }
            match template.save_api_key(&model).await {
                Ok(()) => Json(AjaxJson::ok()),
                Err(e) => Json(AjaxJson::error(e.to_string())),
            }
        }
        Ok(None) => Json(AjaxJson::error("ApiKey 不存在")),
        Err(e) => Json(AjaxJson::error(e.to_string())),
    }
}

/// `/deleteApiKey`
async fn delete_api_key(
    State(state): State<AppState>,
    Query(q): Query<DeleteQuery>,
) -> impl IntoResponse {
    let Ok(login_id) = StpUtil::get_login_id() else {
        return Json(AjaxJson::error("未登录"));
    };
    let template = state.manager.template();
    if let Err(e) = template.check_api_key_login_id(&q.api_key, &login_id).await {
        return Json(AjaxJson::error(e.to_string()));
    }
    match template.delete_api_key(&q.api_key).await {
        Ok(()) => Json(AjaxJson::ok()),
        Err(e) => Json(AjaxJson::error(e.to_string())),
    }
}

/// `/deleteMyAllApiKey`
async fn delete_my_all(State(state): State<AppState>) -> impl IntoResponse {
    let Ok(login_id) = StpUtil::get_login_id() else {
        return Json(AjaxJson::error("未登录"));
    };
    match state
        .manager
        .template()
        .delete_api_key_by_login_id(&login_id)
        .await
    {
        Ok(()) => Json(AjaxJson::ok()),
        Err(e) => Json(AjaxJson::error(e.to_string())),
    }
}

/// 资源接口：必须有效 ApiKey —— `/akRes1`
async fn ak_res1(State(state): State<AppState>, headers: HeaderMap) -> impl IntoResponse {
    let Some(api_key) = read_api_key(&headers, None) else {
        return Json(AjaxJson::error("缺少 apikey"));
    };
    match state.manager.template().check_api_key(&api_key).await {
        Ok(model) => Json(AjaxJson::ok_msg("调用成功").set_data(model)),
        Err(e) => Json(AjaxJson::error(e.to_string())),
    }
}

/// 资源接口：需 userinfo scope —— `/akRes2`
async fn ak_res2(State(state): State<AppState>, headers: HeaderMap) -> impl IntoResponse {
    let Some(api_key) = read_api_key(&headers, None) else {
        return Json(AjaxJson::error("缺少 apikey"));
    };
    match state
        .manager
        .template()
        .check_api_key_scope(&api_key, &["userinfo"])
        .await
    {
        Ok(model) => Json(AjaxJson::ok_msg("调用成功").set_data(model)),
        Err(e) => Json(AjaxJson::error(e.to_string())),
    }
}

/// 资源接口：AND scopes —— `/akRes3`
async fn ak_res3(State(state): State<AppState>, headers: HeaderMap) -> impl IntoResponse {
    let Some(api_key) = read_api_key(&headers, None) else {
        return Json(AjaxJson::error("缺少 apikey"));
    };
    match state
        .manager
        .template()
        .check_api_key_scope(&api_key, &["userinfo", "chat"])
        .await
    {
        Ok(model) => Json(AjaxJson::ok_msg("调用成功").set_data(model)),
        Err(e) => Json(AjaxJson::error(e.to_string())),
    }
}

/// 资源接口：OR scopes —— `/akRes4`
async fn ak_res4(State(state): State<AppState>, headers: HeaderMap) -> impl IntoResponse {
    let Some(api_key) = read_api_key(&headers, None) else {
        return Json(AjaxJson::error("缺少 apikey"));
    };
    match state
        .manager
        .template()
        .check_api_key_scope_or(&api_key, &["userinfo", "chat"])
        .await
    {
        Ok(model) => Json(AjaxJson::ok_msg("调用成功").set_data(model)),
        Err(e) => Json(AjaxJson::error(e.to_string())),
    }
}

#[tokio::main]
async fn main() {
    let state = init();
    let app = Router::new()
        .route("/login", get(login))
        .route("/getLoginId", get(get_login_id))
        .route("/logout", get(logout).post(logout))
        .route("/myApiKeyList", get(my_api_key_list))
        .route("/createApiKey", get(create_api_key).post(create_api_key))
        .route("/updateApiKey", post(update_api_key))
        .route("/deleteApiKey", get(delete_api_key).post(delete_api_key))
        .route("/deleteMyAllApiKey", get(delete_my_all).post(delete_my_all))
        .route("/akRes1", get(ak_res1))
        .route("/akRes2", get(ak_res2))
        .route("/akRes3", get(ak_res3))
        .route("/akRes4", get(ak_res4))
        .with_state(state)
        .layer(SaTokenLayer::new());

    let addr = "0.0.0.0:8091";
    println!("🚀 Sa-Token-Rs Axum ApiKey Demo");
    println!("   http://{addr}");
    println!("   1) GET /login?id=10001");
    println!("   2) GET /createApiKey  (Header: satoken)");
    println!("   3) GET /akRes2        (Header: apikey)");

    let listener = tokio::net::TcpListener::bind(addr).await.expect("bind");
    axum::serve(listener, app).await.expect("serve");
}
