//! Alone-Redis Actix Demo

mod util;

use std::env;
use std::sync::Arc;

use actix_web::{App, HttpRequest, HttpResponse, HttpServer, web};
use redis::AsyncCommands;
use redis::aio::ConnectionManager;
use sa_token::prelude::{AsyncSaTokenRuntime, AsyncStpUtil, SaTokenConfig};
use sa_token_core::context::sa_token_context_default_impl::SaTokenContextDefaultImpl;
use sa_token_dao_redis::SaTokenDaoRedis;
use serde::Deserialize;
use serde_json::json;

use crate::util::AjaxJson;

#[derive(Clone)]
struct AppState {
    util: AsyncStpUtil,
    biz: ConnectionManager,
}

#[derive(Debug, Deserialize)]
struct DoLoginQuery {
    #[serde(default)]
    name: String,
    #[serde(default)]
    pwd: String,
}

#[derive(Debug, Deserialize)]
struct BizQuery {
    #[serde(default)]
    key: String,
    #[serde(default)]
    value: String,
}

fn token_from_req(req: &HttpRequest) -> Option<String> {
    req.headers()
        .get("satoken")?
        .to_str()
        .ok()
        .map(str::to_string)
}

/// 登录。
async fn do_login(state: web::Data<AppState>, q: web::Query<DoLoginQuery>) -> HttpResponse {
    if q.name != "zhang" || q.pwd != "123456" {
        return HttpResponse::Ok().json(AjaxJson::error("登录失败"));
    }
    match state.util.login("10001").await {
        Ok(token) => {
            HttpResponse::Ok().json(AjaxJson::ok_msg("登录成功").set_data(json!({"token": token})))
        }
        Err(e) => HttpResponse::Ok().json(AjaxJson::error(e.to_string())),
    }
}

/// 是否登录。
async fn is_login(state: web::Data<AppState>, req: HttpRequest) -> HttpResponse {
    let logged = match token_from_req(&req) {
        Some(t) => state
            .util
            .get_login_id_by_token(&t)
            .await
            .ok()
            .flatten()
            .is_some(),
        None => false,
    };
    HttpResponse::Ok().json(AjaxJson::ok_data(logged))
}

/// 注销。
async fn logout(state: web::Data<AppState>, req: HttpRequest) -> HttpResponse {
    let Some(token) = token_from_req(&req) else {
        return HttpResponse::Ok().json(AjaxJson::error("未登录"));
    };
    match state.util.get_login_id_by_token(&token).await {
        Ok(Some(id)) => {
            let _ = state.util.logout_by_login_id(&id).await;
            HttpResponse::Ok().json(AjaxJson::ok())
        }
        _ => HttpResponse::Ok().json(AjaxJson::error("token 无效")),
    }
}

/// 业务 SET。
async fn biz_set(state: web::Data<AppState>, q: web::Query<BizQuery>) -> HttpResponse {
    let mut conn = state.biz.clone();
    match conn.set::<_, _, ()>(&q.key, &q.value).await {
        Ok(()) => HttpResponse::Ok().json(AjaxJson::ok_msg("biz set ok")),
        Err(e) => HttpResponse::Ok().json(AjaxJson::error(e.to_string())),
    }
}

/// 业务 GET。
async fn biz_get(state: web::Data<AppState>, q: web::Query<BizQuery>) -> HttpResponse {
    let mut conn = state.biz.clone();
    match conn.get::<_, Option<String>>(&q.key).await {
        Ok(value) => {
            HttpResponse::Ok().json(AjaxJson::ok_data(json!({"key": q.key, "value": value})))
        }
        Err(e) => HttpResponse::Ok().json(AjaxJson::error(e.to_string())),
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let satoken_url = env::var("SA_TOKEN_REDIS_URL")
        .or_else(|_| env::var("REDIS_URL"))
        .unwrap_or_else(|_| "redis://127.0.0.1:6379/0".into());
    let biz_url = env::var("BIZ_REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379/1".into());

    let satoken_client =
        redis::Client::open(satoken_url.as_str()).expect("invalid SA_TOKEN_REDIS_URL");
    let dao = SaTokenDaoRedis::connect(satoken_client)
        .await
        .expect("satoken redis connect");
    let runtime = AsyncSaTokenRuntime::new(
        Arc::new(SaTokenConfig {
            token_name: "satoken".into(),
            timeout: 2_592_000,
            is_log: true,
            ..Default::default()
        }),
        Arc::new(dao),
        Arc::new(SaTokenContextDefaultImpl),
    );
    let util = AsyncStpUtil::new("login", Arc::new(runtime));
    let biz_client = redis::Client::open(biz_url.as_str()).expect("invalid BIZ_REDIS_URL");
    let biz = ConnectionManager::new(biz_client)
        .await
        .expect("biz redis connect");
    let state = AppState { util, biz };

    let addr = ("0.0.0.0", 8114);
    println!("🚀 Alone-Redis Actix Demo  http://{}:{}", addr.0, addr.1);
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(state.clone()))
            .route("/acc/doLogin", web::get().to(do_login))
            .route("/acc/isLogin", web::get().to(is_login))
            .route("/acc/logout", web::get().to(logout))
            .route("/biz/set", web::get().to(biz_set))
            .route("/biz/get", web::get().to(biz_get))
    })
    .bind(addr)?
    .run()
    .await
}
