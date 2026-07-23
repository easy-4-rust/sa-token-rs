//! Sa-Token-Rs Actix Sign Demo
//!
//! 使用 `sa-token-sign` 演示参数签名创建与校验。

mod util;

use std::collections::HashMap;
use std::sync::Arc;

use actix_web::{App, HttpResponse, HttpServer, web};
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
    template: web::Data<Arc<SaSignTemplate>>,
    query: web::Query<CreateQuery>,
) -> HttpResponse {
    let mut params = HashMap::new();
    params.insert("userId".into(), query.user_id.clone());
    params.insert("amount".into(), query.amount.clone());
    match template.add_sign_params(&mut params) {
        Ok(()) => HttpResponse::Ok().json(AjaxJson::ok_data(params)),
        Err(e) => HttpResponse::Ok().json(AjaxJson::error(e.to_string())),
    }
}

/// 校验签名 —— `/sign/check`
async fn sign_check(
    template: web::Data<Arc<SaSignTemplate>>,
    query: web::Query<HashMap<String, String>>,
) -> HttpResponse {
    match template.check_param_map(&query) {
        Ok(()) => HttpResponse::Ok().json(AjaxJson::ok_msg("签名校验通过")),
        Err(e) => HttpResponse::Ok().json(AjaxJson::error(e.to_string())),
    }
}

/// 受保护下单接口 —— `/api/order`
async fn api_order(
    template: web::Data<Arc<SaSignTemplate>>,
    query: web::Query<HashMap<String, String>>,
) -> HttpResponse {
    match template.check_param_map(&query) {
        Ok(()) => HttpResponse::Ok().json(AjaxJson::ok_data(json!({
            "msg": "下单成功",
            "userId": query.get("userId"),
            "amount": query.get("amount"),
        }))),
        Err(e) => HttpResponse::Ok().json(AjaxJson::error(format!("签名校验失败：{e}"))),
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let template = build_template();
    let addr = ("0.0.0.0", 8120);

    println!("🚀 Sa-Token-Rs Actix Sign Demo");
    println!("   http://{}:{}", addr.0, addr.1);
    println!("   1) GET /sign/create?userId=10001&amount=100");
    println!("   2) 将返回的参数原样拼到 /sign/check 或 /api/order");

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(template.clone()))
            .route("/sign/create", web::get().to(sign_create))
            .route("/sign/check", web::get().to(sign_check))
            .route("/api/order", web::get().to(api_order))
    })
    .bind(addr)?
    .run()
    .await
}
