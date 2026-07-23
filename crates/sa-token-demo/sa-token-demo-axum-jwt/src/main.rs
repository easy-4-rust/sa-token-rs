//! Sa-Token-Rs Axum JWT Demo
//!
//! 对应 Java：`sa-token-demo-jwt`（Spring Boot → axum，Jackson → serde）

mod controllers;
mod satoken;
mod util;

use std::sync::Arc;

use axum::Router;
use axum::routing::get;
use sa_token_web_axum::SaTokenLayer;

use crate::controllers::test_controller;
use crate::satoken::init_sa_token;

#[tokio::main]
async fn main() {
    let jwt = Arc::new(init_sa_token());

    let app = Router::new()
        .route("/test/login", get(test_controller::login))
        .route("/test/tokenInfo", get(test_controller::token_info))
        .route("/test/session", get(test_controller::session))
        .route("/test/test", get(test_controller::test))
        .with_state(jwt)
        .layer(SaTokenLayer::new());

    let addr = "0.0.0.0:8083";
    println!("🚀 Sa-Token-Rs Axum JWT Demo（对应 sa-token-demo-jwt）");
    println!("   http://{addr}");
    println!("   GET /test/login?id=10001");

    let listener = tokio::net::TcpListener::bind(addr).await.expect("bind");
    axum::serve(listener, app).await.expect("serve");
}
