//! Sa-Token-Rs Axum Demo
//!
//! 对应 Java：`sa-token-demo-springboot`
//! 框架映射：Spring Boot → axum，Jackson → serde

mod controllers;
mod current;
mod satoken;
mod util;

use axum::Router;
use axum::http::{HeaderName, HeaderValue};
use axum::routing::get;
use sa_token_web_axum::SaTokenLayer;
use tower_http::set_header::SetResponseHeaderLayer;

use crate::controllers::{at_controller, login_controller, test_controller};
use crate::satoken::init_sa_token;

/// 组装路由（对应 Java Controller 注册）。
fn build_router() -> Router {
    Router::new()
        // LoginController → /acc/**
        .route("/acc/doLogin", get(login_controller::do_login).post(login_controller::do_login))
        .route("/acc/isLogin", get(login_controller::is_login))
        .route("/acc/tokenInfo", get(login_controller::token_info))
        .route("/acc/logout", get(login_controller::logout).post(login_controller::logout))
        // TestController → /test/**
        .route("/test/login", get(test_controller::login))
        .route("/test/logout", get(test_controller::logout))
        .route("/test/testRole", get(test_controller::test_role))
        .route("/test/testJur", get(test_controller::test_jur))
        .route("/test/session", get(test_controller::session))
        .route("/test/session2", get(test_controller::session2))
        .route("/test/getTokenSession", get(test_controller::get_token_session))
        .route("/test/tokenInfo", get(test_controller::token_info))
        .route("/test/atCheck", get(test_controller::at_check))
        .route("/test/atJurOr", get(test_controller::at_jur_or))
        .route("/test/rene", get(test_controller::rene))
        .route("/test/kickOut", get(test_controller::kick_out))
        .route("/test/login2", get(test_controller::login2))
        .route("/test/switchTo", get(test_controller::switch_to))
        .route("/test/loginByDevice", get(test_controller::login_by_device))
        .route("/test/test", get(test_controller::test))
        .route("/test/test2", get(test_controller::test2))
        // AtController → /at/**
        .route("/at/checkLogin", get(at_controller::check_login))
        .route("/at/checkPermission", get(at_controller::check_permission))
        .route(
            "/at/checkPermissionAnd",
            get(at_controller::check_permission_and),
        )
        .route(
            "/at/checkPermissionOr",
            get(at_controller::check_permission_or),
        )
        .route("/at/checkRole", get(at_controller::check_role))
        .route("/at/openSafe", get(at_controller::open_safe).post(at_controller::open_safe))
        .route("/at/checkSafe", get(at_controller::check_safe))
        // 安全响应头（对应 SaServletFilter#setBeforeAuth）
        .layer(SetResponseHeaderLayer::overriding(
            HeaderName::from_static("x-frame-options"),
            HeaderValue::from_static("SAMEORIGIN"),
        ))
        .layer(SetResponseHeaderLayer::overriding(
            HeaderName::from_static("x-xss-protection"),
            HeaderValue::from_static("1; mode=block"),
        ))
        .layer(SetResponseHeaderLayer::overriding(
            HeaderName::from_static("x-content-type-options"),
            HeaderValue::from_static("nosniff"),
        ))
        .layer(SetResponseHeaderLayer::overriding(
            HeaderName::from_static("server"),
            HeaderValue::from_static("sa-server"),
        ))
        .layer(SaTokenLayer::new())
}

#[tokio::main]
async fn main() {
    init_sa_token();

    let app = build_router();
    let addr = "0.0.0.0:8081";
    println!("🚀 Sa-Token-Rs Axum Demo（对应 sa-token-demo-springboot）");
    println!("   http://{addr}");
    println!("   示例: GET /acc/doLogin?name=zhang&pwd=123456");
    println!("   登录后请求头携带: satoken: <token>");

    let listener = tokio::net::TcpListener::bind(addr).await.expect("bind");
    axum::serve(listener, app).await.expect("serve");
}
