//! Sa-Token-Rs Actix Redis Demo
//!
//! 框架映射：Quarkus → actix-web；对应 Java `sa-token-demo-springboot-redis`

mod util;

use std::env;
use std::sync::Arc;

use actix_web::{App, HttpRequest, HttpResponse, HttpServer, web};
use sa_token::prelude::{AsyncSaTokenRuntime, AsyncStpUtil, SaTokenConfig};
use sa_token_web_actix::{RequireLogin, require_login};
use sa_token_core::context::sa_token_context_default_impl::SaTokenContextDefaultImpl;
use sa_token_core::stp::StpInterface;
use sa_token_dao_redis::SaTokenDaoRedis;
use serde::Deserialize;
use serde_json::json;

use crate::util::AjaxJson;

/// 权限数据源。
struct StpInterfaceImpl;

impl StpInterface for StpInterfaceImpl {
    fn get_permission_list(&self, _login_id: &str, _login_type: &str) -> Vec<String> {
        vec!["101".into(), "user-add".into(), "user-delete".into()]
    }

    fn get_role_list(&self, _login_id: &str, _login_type: &str) -> Vec<String> {
        vec!["admin".into(), "super-admin".into()]
    }
}

/// 登录参数。
#[derive(Debug, Deserialize)]
struct DoLoginQuery {
    #[serde(default)]
    name: String,
    #[serde(default)]
    pwd: String,
}

/// 登录 id。
#[derive(Debug, Deserialize)]
struct LoginQuery {
    #[serde(default = "default_id")]
    id: String,
}

fn default_id() -> String {
    "10001".into()
}

/// 构建 AsyncStpUtil（Redis）。
async fn build_util() -> AsyncStpUtil {
    let redis_url = env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".into());
    println!("Connecting Redis: {redis_url}");
    let client = redis::Client::open(redis_url.as_str()).expect("invalid REDIS_URL");
    let dao = SaTokenDaoRedis::connect(client)
        .await
        .expect("redis connect failed");

    let runtime = AsyncSaTokenRuntime::new(
        Arc::new(SaTokenConfig {
            token_name: "satoken".into(),
            timeout: 2_592_000,
            is_concurrent: true,
            is_share: false,
            is_log: true,
            ..Default::default()
        }),
        Arc::new(dao),
        Arc::new(SaTokenContextDefaultImpl),
    )
    .with_stp_interface(Arc::new(StpInterfaceImpl));

    AsyncStpUtil::new("login", Arc::new(runtime))
}

/// `/acc/doLogin`
async fn do_login(util: web::Data<AsyncStpUtil>, query: web::Query<DoLoginQuery>) -> HttpResponse {
    if query.name == "zhang" && query.pwd == "123456" {
        match util.login("10001").await {
            Ok(token) => HttpResponse::Ok()
                .json(AjaxJson::ok_msg("登录成功").set_data(json!({ "token": token }))),
            Err(e) => HttpResponse::Ok().json(AjaxJson::error(e.to_string())),
        }
    } else {
        HttpResponse::Ok().json(AjaxJson::error("登录失败"))
    }
}

/// `/test/login`
async fn test_login(util: web::Data<AsyncStpUtil>, query: web::Query<LoginQuery>) -> HttpResponse {
    match util.login(&query.id).await {
        Ok(token) => HttpResponse::Ok().json(AjaxJson::ok().set_data(json!({ "token": token }))),
        Err(e) => HttpResponse::Ok().json(AjaxJson::error(e.to_string())),
    }
}

/// `/acc/isLogin`
async fn is_login(util: web::Data<AsyncStpUtil>, req: HttpRequest) -> HttpResponse {
    let token = req
        .headers()
        .get("satoken")
        .and_then(|v| v.to_str().ok())
        .map(str::to_string);
    let logged_in = match token {
        Some(t) => util
            .get_login_id_by_token(&t)
            .await
            .ok()
            .flatten()
            .is_some(),
        None => false,
    };
    HttpResponse::Ok().json(AjaxJson::ok_msg(format!("是否登录：{logged_in}")))
}

/// `/secure/acc/logout`
async fn logout(util: web::Data<AsyncStpUtil>, login: RequireLogin) -> HttpResponse {
    match util.logout_by_login_id(&login.0.login_id).await {
        Ok(()) => HttpResponse::Ok().json(AjaxJson::ok()),
        Err(e) => HttpResponse::Ok().json(AjaxJson::error(e.to_string())),
    }
}

/// `/secure/test/tokenInfo`
async fn token_info(login: RequireLogin) -> HttpResponse {
    HttpResponse::Ok().json(AjaxJson::ok_data(json!({
        "login_id": login.0.login_id,
        "token_value": login.0.token,
    })))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let util = build_util().await;
    let addr = ("0.0.0.0", 8086);

    println!("🚀 Sa-Token-Rs Actix Redis Demo（Quarkus → actix）");
    println!("   http://{}:{}", addr.0, addr.1);

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(util.clone()))
            .route("/acc/doLogin", web::get().to(do_login))
            .route("/acc/isLogin", web::get().to(is_login))
            .route("/test/login", web::get().to(test_login))
            .service(
                web::scope("/secure")
                    .wrap(actix_web::middleware::from_fn(require_login))
                    .route("/acc/logout", web::get().to(logout))
                    .route("/test/tokenInfo", web::get().to(token_info)),
            )
    })
    .bind(addr)?
    .run()
    .await
}
