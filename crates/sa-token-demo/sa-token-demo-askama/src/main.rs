//! Sa-Token Thymeleaf 方言 Demo（对应 Java `sa-token-demo-thymeleaf`）。
//!
//! 框架映射：Spring Boot → axum；模板：Thymeleaf → Askama；JSON：Jackson → serde。

mod controllers;
mod current;
mod satoken;
mod util;
mod views;

use std::sync::Arc;

use axum::Router;
use axum::routing::get;
use sa_token_askama::SaTokenDialect;
use sa_token_web_axum::SaTokenLayer;

use crate::controllers::test_controller;
use crate::satoken::configure::init_sa_token;

/// 应用入口（对应 Java `SaTokenThymeleafDemoApplication.main`）。
#[tokio::main]
async fn main() {
    // 初始化 Sa-Token（对应 Java Spring 自动配置）
    let stp = init_sa_token();

    // 构建 Askama 方言（对应 Java Thymeleaf SaTokenDialect 注册）
    let dialect = Arc::new(SaTokenDialect::new(stp));

    let app = Router::new()
        .route("/", get(test_controller::index))
        .route("/login", get(test_controller::login))
        .route("/logout", get(test_controller::logout))
        .with_state(dialect)
        .layer(SaTokenLayer::new());

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8082")
        .await
        .expect("bind 8082");
    println!("sa-token-demo-askama listening on http://127.0.0.1:8082");
    axum::serve(listener, app).await.expect("serve");
}
