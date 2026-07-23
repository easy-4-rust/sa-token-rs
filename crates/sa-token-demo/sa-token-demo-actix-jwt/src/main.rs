//! Sa-Token-Rs Actix JWT Demo
//!
//! 框架映射：Quarkus → actix-web；对应 Java `sa-token-demo-jwt`

mod util;

use std::collections::HashMap;
use std::sync::Arc;

use actix_web::{App, HttpResponse, HttpServer, web};
use sa_token::prelude::{
    AsyncSaTokenRuntime, AsyncStpUtil, SaLoginParameter, SaTokenConfig, SaTokenDaoMemory,
};
use sa_token_core::context::sa_token_context_default_impl::SaTokenContextDefaultImpl;
use sa_token_jwt::StpLogicJwtForSimple;
use serde::Deserialize;
use serde_json::json;

use crate::util::AjaxJson;

/// JWT 密钥。
const JWT_SECRET: &str = "asdhfjkasdhfjkashdfjkashdfkjahsdjkfhaskldjf";

/// 应用状态。
#[derive(Clone)]
struct AppState {
    /// Async 门面
    util: AsyncStpUtil,
    /// JWT Simple
    jwt: Arc<StpLogicJwtForSimple>,
}

/// 登录参数。
#[derive(Debug, Deserialize)]
struct LoginQuery {
    /// 账号 id
    #[serde(default = "default_id")]
    id: String,
}

fn default_id() -> String {
    "10001".into()
}

/// 构建状态。
fn build_state() -> AppState {
    let runtime = AsyncSaTokenRuntime::new(
        Arc::new(SaTokenConfig {
            token_name: "satoken".into(),
            timeout: 2_592_000,
            is_log: true,
            ..Default::default()
        }),
        Arc::new(SaTokenDaoMemory::new()),
        Arc::new(SaTokenContextDefaultImpl),
    );
    AppState {
        util: AsyncStpUtil::new("login", Arc::new(runtime)),
        jwt: Arc::new(StpLogicJwtForSimple::new("login", JWT_SECRET)),
    }
}

/// 登录 —— `/test/login`
async fn login(state: web::Data<AppState>, query: web::Query<LoginQuery>) -> HttpResponse {
    let mut extra = HashMap::new();
    extra.insert("name".into(), json!("张三"));

    let param = SaLoginParameter::default().set_extra_data(json!({ "name": "张三" }));
    match state.util.login_with_param(&query.id, &param).await {
        Ok(session_token) => match state.jwt.create_token_value(json!(query.id), extra) {
            Ok(jwt_token) => HttpResponse::Ok().json(AjaxJson::ok_data(json!({
                "session_token": session_token,
                "jwt_token": jwt_token,
            }))),
            Err(e) => HttpResponse::Ok().json(AjaxJson::error(e.to_string())),
        },
        Err(e) => HttpResponse::Ok().json(AjaxJson::error(e.to_string())),
    }
}

/// 解析 JWT —— `/test/parseJwt?token=xxx`
async fn parse_jwt(
    state: web::Data<AppState>,
    query: web::Query<HashMap<String, String>>,
) -> HttpResponse {
    let Some(token) = query.get("token") else {
        return HttpResponse::Ok().json(AjaxJson::error("缺少 token 参数"));
    };
    match state.jwt.get_extra(token, "name") {
        Ok(name) => HttpResponse::Ok().json(AjaxJson::ok_data(json!({ "name": name }))),
        Err(e) => HttpResponse::Ok().json(AjaxJson::error(e.to_string())),
    }
}

/// 健康探针 —— `/test/test`
async fn test() -> HttpResponse {
    HttpResponse::Ok().json(AjaxJson::ok())
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let state = build_state();
    let addr = ("0.0.0.0", 8084);
    println!("🚀 Sa-Token-Rs Actix JWT Demo（Quarkus → actix）");
    println!("   http://{}:{}", addr.0, addr.1);

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(state.clone()))
            .route("/test/login", web::get().to(login))
            .route("/test/parseJwt", web::get().to(parse_jwt))
            .route("/test/test", web::get().to(test))
    })
    .bind(addr)?
    .run()
    .await
}
