//! Sa-Token-Rs Axum Sign Demo
//!
//! 使用 `sa-token-sign` 演示参数签名创建与校验。

mod util;

use std::collections::HashMap;
use std::sync::Arc;

use axum::Json;
use axum::Router;
use axum::extract::{Query, State};
use axum::routing::get;
use sa_token_core::dao::sa_token_dao::SaTokenDao;
use sa_token_dao_memory::SaTokenDaoMemory;
use sa_token_sign::sign::{SaSignConfig, SaSignTemplate};
use serde::Deserialize;
use serde_json::json;

use crate::util::AjaxJson;

/// 创建签名参数。
#[derive(Debug, Deserialize)]
struct CreateQuery {
    #[serde(default, rename = "userId")]
    user_id: String,
    #[serde(default)]
    amount: String,
}

/// 构建签名模板。
fn build_template() -> Arc<SaSignTemplate> {
    let dao: Arc<dyn SaTokenDao> = Arc::new(SaTokenDaoMemory::new());
    Arc::new(SaSignTemplate::new(
        Arc::new(SaSignConfig::new("demo-secret-key")),
        dao,
        "satoken",
    ))
}

/// 创建签名 —— `/sign/create`
async fn sign_create(
    State(template): State<Arc<SaSignTemplate>>,
    Query(q): Query<CreateQuery>,
) -> Json<AjaxJson> {
    let mut params = HashMap::new();
    params.insert("userId".into(), q.user_id);
    params.insert("amount".into(), q.amount);
    match template.add_sign_params(&mut params) {
        Ok(()) => Json(AjaxJson::ok_data(params)),
        Err(e) => Json(AjaxJson::error(e.to_string())),
    }
}

/// 校验签名 —— `/sign/check`
async fn sign_check(
    State(template): State<Arc<SaSignTemplate>>,
    Query(params): Query<HashMap<String, String>>,
) -> Json<AjaxJson> {
    match template.check_param_map(&params) {
        Ok(()) => Json(AjaxJson::ok_msg("签名校验通过")),
        Err(e) => Json(AjaxJson::error(e.to_string())),
    }
}

/// 受保护下单接口 —— `/api/order`
async fn api_order(
    State(template): State<Arc<SaSignTemplate>>,
    Query(params): Query<HashMap<String, String>>,
) -> Json<AjaxJson> {
    match template.check_param_map(&params) {
        Ok(()) => Json(AjaxJson::ok_data(json!({
            "msg": "下单成功",
            "userId": params.get("userId"),
            "amount": params.get("amount"),
        }))),
        Err(e) => Json(AjaxJson::error(format!("签名校验失败：{e}"))),
    }
}

#[tokio::main]
async fn main() {
    let template = build_template();

    let app = Router::new()
        .route("/sign/create", get(sign_create))
        .route("/sign/check", get(sign_check))
        .route("/api/order", get(api_order))
        .with_state(template);

    let addr = "0.0.0.0:8119";
    println!("🚀 Sa-Token-Rs Axum Sign Demo");
    println!("   http://{addr}");
    println!("   1) GET /sign/create?userId=10001&amount=100");
    println!("   2) 将返回的参数原样拼到 /sign/check 或 /api/order");

    let listener = tokio::net::TcpListener::bind(addr).await.expect("bind");
    axum::serve(listener, app).await.expect("serve");
}
