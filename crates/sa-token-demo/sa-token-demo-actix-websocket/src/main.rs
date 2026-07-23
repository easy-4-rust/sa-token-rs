//! Sa-Token-Rs Actix WebSocket Demo
//!
//! 对应 Java：`sa-token-demo-websocket`（WebSocketConnect，AsyncStpUtil + actix-ws）

mod util;

use std::sync::Arc;

use actix_web::{App, Error, HttpRequest, HttpResponse, HttpServer, web};
use actix_ws::Message;
use sa_token::prelude::{AsyncSaTokenRuntime, AsyncStpUtil, SaTokenConfig};
use sa_token_core::context::sa_token_context_default_impl::SaTokenContextDefaultImpl;
use sa_token_dao_memory::SaTokenDaoMemory;
use serde::Deserialize;

use crate::util::AjaxJson;

/// 应用状态。
#[derive(Clone)]
struct AppState {
    util: AsyncStpUtil,
}

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

/// 初始化 AsyncStpUtil。
fn build_state() -> AppState {
    let dao = Arc::new(SaTokenDaoMemory::new());
    let runtime = AsyncSaTokenRuntime::new(
        Arc::new(SaTokenConfig::default()),
        Arc::clone(&dao) as Arc<_>,
        Arc::new(SaTokenContextDefaultImpl),
    );
    AppState {
        util: AsyncStpUtil::new("login", Arc::new(runtime)),
    }
}

/// 登录 —— `/acc/doLogin`
async fn do_login(state: web::Data<AppState>, query: web::Query<LoginQuery>) -> HttpResponse {
    if query.name == "zhang" && query.pwd == "123456" {
        match state.util.login("10001").await {
            Ok(token) => HttpResponse::Ok().json(AjaxJson::ok().set("satoken", token)),
            Err(e) => HttpResponse::Ok().json(AjaxJson::error(e.to_string())),
        }
    } else {
        HttpResponse::Ok().json(AjaxJson::error("账号名或密码错误"))
    }
}

/// WebSocket 握手 —— `/ws-connect/{satoken}`
///
/// 连接时校验 Token，无效则拒绝；成功后发送欢迎语并回显消息。
async fn ws_connect(
    req: HttpRequest,
    stream: web::Payload,
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> Result<HttpResponse, Error> {
    let satoken = path.into_inner();
    let login_id = match state.util.get_login_id_by_token(&satoken).await {
        Ok(Some(id)) => id,
        Ok(None) => {
            return Ok(HttpResponse::Unauthorized().body("invalid satoken"));
        }
        Err(e) => {
            return Ok(HttpResponse::Unauthorized().body(e.to_string()));
        }
    };

    let (response, mut session, mut msg_stream) = actix_ws::handle(&req, stream)?;

    actix_web::rt::spawn(async move {
        let welcome = format!("welcome, loginId={login_id}");
        if session.text(welcome).await.is_err() {
            return;
        }

        while let Some(Ok(msg)) = msg_stream.recv().await {
            match msg {
                Message::Text(text) => {
                    if session.text(text).await.is_err() {
                        break;
                    }
                }
                Message::Binary(bin) => {
                    if session.binary(bin).await.is_err() {
                        break;
                    }
                }
                Message::Ping(bytes) => {
                    if session.pong(&bytes).await.is_err() {
                        break;
                    }
                }
                Message::Close(_) => break,
                Message::Pong(_) | Message::Continuation(_) | Message::Nop => {}
            }
        }

        let _ = session.close(None).await;
    });

    Ok(response)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let state = build_state();
    let addr = ("0.0.0.0", 8098);
    println!("🚀 Sa-Token-Rs Actix WebSocket Demo");
    println!("   http://{}:{}", addr.0, addr.1);
    println!(
        "   login: zhang / 123456 → ws://{}:{}/ws-connect/{{satoken}}",
        addr.0, addr.1
    );

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(state.clone()))
            .app_data(web::Data::new(state.util.clone()))
            .route("/acc/doLogin", web::get().to(do_login))
            .route("/ws-connect/{satoken}", web::get().to(ws_connect))
    })
    .bind(addr)?
    .run()
    .await
}
