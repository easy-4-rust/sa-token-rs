//! Sa-Token Freemarker 方言 Demo（对应 Java `sa-token-demo-freemarker`）。
//!
//! 框架映射：Spring Boot → axum；模板：FreeMarker → Tera；JSON：Jackson → serde。

mod controllers;
mod current;
mod satoken;
mod util;

use std::sync::Arc;

use axum::Router;
use axum::routing::get;
use sa_token_web_axum::SaTokenLayer;
use tera::Tera;

use crate::controllers::test_controller;
use crate::satoken::configure::init_sa_token;

/// 应用入口（对应 Java `SaTokenFreemarkerDemoApplication.main`）。
#[tokio::main]
async fn main() {
    // 初始化 Sa-Token（对应 Java Spring 自动配置）
    let stp = init_sa_token();

    // 构建 Tera 并注册 Sa-Token 方言函数（对应 Java Freemarker 方言注册）
    let mut tera = Tera::new("templates/**/*").unwrap_or_else(|e| {
        eprintln!("模板加载失败: {e}");
        Tera::default()
    });
    sa_token_tera::SaTokenTemplateModel::new(stp).register_into(&mut tera);
    let tera = Arc::new(tera);

    let app = Router::new()
        .route("/", get(test_controller::index))
        .route("/login", get(test_controller::login))
        .route("/logout", get(test_controller::logout))
        .with_state(tera)
        .layer(SaTokenLayer::new());

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8081")
        .await
        .expect("bind 8081");
    println!("sa-token-demo-tera listening on http://127.0.0.1:8081");
    axum::serve(listener, app).await.expect("serve");
}
