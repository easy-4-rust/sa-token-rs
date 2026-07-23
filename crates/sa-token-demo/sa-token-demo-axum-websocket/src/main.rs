//! Sa-Token-Rs Axum WebSocket Demo
//!
//! 对应 Java：`sa-token-demo-websocket`（WebSocketConnect）

mod util;

use std::sync::Arc;

use axum::Json;
use axum::Router;
use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::{Path, Query};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::get;
use sa_token::prelude::*;
use sa_token_web_axum::SaTokenLayer;
use sa_token_dao_memory::SaTokenDaoMemory;
use serde::Deserialize;

use crate::util::AjaxJson;

#[derive(Debug, Deserialize)]
struct LoginQuery {
    #[serde(default = "default_name")]
    name: String,
    #[serde(default = "default_pwd")]
    pwd: String,
}

fn default_name() -> String {
    "zhang".into()
}
fn default_pwd() -> String {
    "123456".into()
}

/// 初始化全局 StpUtil。
fn init() {
    SaManager::set_config(Arc::new(SaTokenConfig::default()));
    SaManager::set_sa_token_dao(Arc::new(SaTokenDaoMemory::new()));
    SaManager::put_stp_logic(Arc::new(StpLogic::new("login")));
}

/// 登录 —— `/acc/doLogin`
async fn do_login(Query(q): Query<LoginQuery>) -> impl IntoResponse {
    if q.name == "zhang" && q.pwd == "123456" {
        match StpUtil::login("10001") {
            Ok(()) => {
                let token = StpUtil::get_token_value().unwrap_or_default();
                Json(AjaxJson::ok().set("satoken", token))
            }
            Err(e) => Json(AjaxJson::error(e.to_string())),
        }
    } else {
        Json(AjaxJson::error("账号名或密码错误"))
    }
}

/// WebSocket 握手 —— `/ws-connect/{satoken}`
///
/// 连接时校验 Token，无效则拒绝；成功后发送欢迎语并回显消息。
async fn ws_connect(ws: WebSocketUpgrade, Path(satoken): Path<String>) -> impl IntoResponse {
    match StpUtil::get_login_id_by_token(&satoken) {
        Ok(Some(login_id)) => ws.on_upgrade(move |socket| handle_socket(socket, login_id)),
        Ok(None) => (StatusCode::UNAUTHORIZED, "invalid satoken").into_response(),
        Err(e) => (StatusCode::UNAUTHORIZED, e.to_string()).into_response(),
    }
}

/// 处理已鉴权的 WebSocket 会话：欢迎 + echo。
async fn handle_socket(mut socket: WebSocket, login_id: String) {
    let welcome = format!("welcome, loginId={login_id}");
    if socket.send(Message::Text(welcome.into())).await.is_err() {
        return;
    }

    while let Some(Ok(msg)) = socket.recv().await {
        match msg {
            Message::Text(text) => {
                if socket.send(Message::Text(text)).await.is_err() {
                    break;
                }
            }
            Message::Binary(bin) => {
                if socket.send(Message::Binary(bin)).await.is_err() {
                    break;
                }
            }
            Message::Ping(payload) => {
                if socket.send(Message::Pong(payload)).await.is_err() {
                    break;
                }
            }
            Message::Close(_) => break,
            Message::Pong(_) => {}
        }
    }
}

#[tokio::main]
async fn main() {
    init();
    let app = Router::new()
        .route("/acc/doLogin", get(do_login))
        .route("/ws-connect/{satoken}", get(ws_connect))
        .layer(SaTokenLayer::new());

    let addr = "0.0.0.0:8097";
    println!("🚀 Sa-Token-Rs Axum WebSocket Demo");
    println!("   http://{addr}");
    println!("   login: zhang / 123456 → ws://{addr}/ws-connect/{{satoken}}");

    let listener = tokio::net::TcpListener::bind(addr).await.expect("bind");
    axum::serve(listener, app).await.expect("serve");
}
