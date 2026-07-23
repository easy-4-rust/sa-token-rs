//! Sa-Token-Rs Axum SSO Server Mini Demo
//!
//! 对应 Java：`sa-token-demo-sso-server`（精简版）

mod util;

use std::sync::Arc;

use axum::Json;
use axum::extract::{Query, State};
use axum::routing::get;
use axum::{Router, response::IntoResponse};
use sa_token::prelude::*;
use sa_token_web_axum::SaTokenLayer;
use sa_token_core::dao::sa_token_dao::SaTokenDao;
use sa_token_dao_memory::SaTokenDaoMemory;
use sa_token_sign::sign::SaSignConfig;
use sa_token_sso::sso::config::{SaSsoClientModel, SaSsoServerConfig};
use sa_token_sso::sso::exception::SaSsoException;
use sa_token_sso::sso::strategy::SaSsoServerStrategy;
use sa_token_sso::sso::template::{SaSsoServerAuth, SaSsoServerTemplate};
use serde::Deserialize;
use serde_json::{Value, json};

use crate::util::AjaxJson;

/// 将 JSON loginId 转为字符串。
fn value_as_login_id(login_id: &Value) -> String {
    match login_id {
        Value::String(s) => s.clone(),
        Value::Number(n) => n.to_string(),
        other => other.to_string(),
    }
}

/// 基于同步 StpUtil 的 SSO Server Auth 端口。
struct DemoServerAuth;

impl SaSsoServerAuth for DemoServerAuth {
    /// 按 Token 查询登录设备 ID。
    fn login_device_id_by_token(
        &self,
        token_value: &str,
    ) -> Result<Option<String>, SaSsoException> {
        StpUtil::stp_logic()
            .get_login_device_id_by_token(token_value)
            .map_err(|e| SaSsoException::new(0, e.to_string()))
    }

    /// 按 Token 查询剩余有效期（秒）。
    fn token_timeout(&self, token_value: &str) -> Result<i64, SaSsoException> {
        StpUtil::stp_logic()
            .get_token_timeout_by_token(token_value)
            .map_err(|e| SaSsoException::new(0, e.to_string()))
    }

    /// 按 loginId 查询 Account-Session 剩余有效期（秒）。
    fn session_timeout(&self, login_id: &Value) -> Result<i64, SaSsoException> {
        let id = value_as_login_id(login_id);
        let session_key = StpUtil::stp_logic().splicing_key_session(&id);
        SaManager::sa_token_dao()
            .get_session_timeout(&session_key)
            .map_err(|e| SaSsoException::new(0, e.to_string()))
    }

    /// 注销指定账号会话。
    fn logout(&self, login_id: &Value, _device_id: Option<String>) -> Result<(), SaSsoException> {
        let id = value_as_login_id(login_id);
        StpUtil::logout_by_login_id(&id).map_err(|e| SaSsoException::new(0, e.to_string()))
    }
}

/// 应用状态。
#[derive(Clone)]
struct AppState {
    template: Arc<SaSsoServerTemplate>,
}

#[derive(Debug, Deserialize)]
struct LoginQuery {
    #[serde(default)]
    name: String,
    #[serde(default)]
    pwd: String,
}

#[derive(Debug, Deserialize)]
struct AuthQuery {
    #[serde(default = "default_client")]
    client: String,
    #[serde(default)]
    redirect: String,
}

fn default_client() -> String {
    "sso-client".into()
}

#[derive(Debug, Deserialize)]
struct CheckTicketQuery {
    ticket: String,
    #[serde(default = "default_client")]
    client: String,
}

/// 初始化 SSO Server 与全局 StpUtil。
fn build_state() -> AppState {
    let dao = Arc::new(SaTokenDaoMemory::new());
    SaManager::set_config(Arc::new(SaTokenConfig::default()));
    SaManager::set_sa_token_dao(Arc::clone(&dao) as Arc<dyn SaTokenDao>);
    SaManager::put_stp_logic(Arc::new(StpLogic::new("login")));

    let mut config = SaSsoServerConfig {
        is_check_sign: false,
        allow_anon_client: true,
        allow_url: "*".into(),
        home_route: Some("/".into()),
        ..Default::default()
    };
    config.add_client(SaSsoClientModel {
        client: "sso-client".into(),
        allow_url: "*".into(),
        ..Default::default()
    });

    let mut strategy = SaSsoServerStrategy::default();
    // 账号密码登录策略：sa / 123456 → 登录 10001。
    strategy.do_login_handle = Arc::new(|name, pwd| {
        if name == "sa" && pwd == "123456" {
            match StpUtil::login("10001") {
                Ok(()) => {
                    let token = StpUtil::get_token_value().unwrap_or_default();
                    json!({"code": 200, "msg": "ok", "data": {"satoken": token}})
                }
                Err(e) => json!({"code": 500, "msg": e.to_string()}),
            }
        } else {
            json!({"code": 500, "msg": "账号名或密码错误"})
        }
    });

    let template = SaSsoServerTemplate::new(
        Arc::new(config),
        Arc::new(strategy),
        Arc::new(SaSignConfig::default()),
        Arc::clone(&dao) as Arc<dyn SaTokenDao>,
        Arc::new(DemoServerAuth),
        "satoken",
    )
    .expect("sso server template");

    AppState {
        template: Arc::new(template),
    }
}

/// SSO 登录 —— `/sso/doLogin`
async fn do_login(State(state): State<AppState>, Query(q): Query<LoginQuery>) -> impl IntoResponse {
    let result = (state.template.strategy.do_login_handle)(&q.name, &q.pwd);
    Json(result)
}

/// 授权下发 ticket —— `/sso/auth`
async fn sso_auth(State(state): State<AppState>, Query(q): Query<AuthQuery>) -> impl IntoResponse {
    let Ok(true) = StpUtil::is_login() else {
        return Json(AjaxJson::error("未登录，请先调用 /sso/doLogin"));
    };
    let Ok(login_id) = StpUtil::get_login_id() else {
        return Json(AjaxJson::error("未登录，请先调用 /sso/doLogin"));
    };
    let Some(token_value) = StpUtil::get_token_value() else {
        return Json(AjaxJson::error("缺少 Token"));
    };
    if q.redirect.is_empty() {
        return Json(AjaxJson::error("缺少 redirect 参数"));
    }
    match state
        .template
        .build_redirect_url(&q.client, &q.redirect, json!(login_id), &token_value)
    {
        Ok(url) => Json(AjaxJson::ok_data(json!({ "redirect": url }))),
        Err(e) => Json(AjaxJson::error(e.to_string())),
    }
}

/// 校验并消费 ticket —— `/sso/checkTicket`
async fn check_ticket(
    State(state): State<AppState>,
    Query(q): Query<CheckTicketQuery>,
) -> impl IntoResponse {
    match state.template.check_ticket_and_delete(&q.ticket, &q.client) {
        Ok(model) => Json(AjaxJson::ok_data(model.login_id)),
        Err(e) => Json(AjaxJson::error(e.to_string())),
    }
}

/// 是否已登录 —— `/acc/isLogin`
async fn is_login() -> impl IntoResponse {
    match StpUtil::is_login() {
        Ok(flag) => Json(AjaxJson::ok_data(flag)),
        Err(e) => Json(AjaxJson::error(e.to_string())),
    }
}

#[tokio::main]
async fn main() {
    let state = build_state();
    let app = Router::new()
        .route("/sso/doLogin", get(do_login))
        .route("/sso/auth", get(sso_auth))
        .route("/sso/checkTicket", get(check_ticket))
        .route("/acc/isLogin", get(is_login))
        .with_state(state)
        .layer(SaTokenLayer::new());

    let addr = "0.0.0.0:8095";
    println!("🚀 Sa-Token-Rs Axum SSO Server Demo");
    println!("   http://{addr}");
    println!("   client=sso-client  login: sa / 123456");

    let listener = tokio::net::TcpListener::bind(addr).await.expect("bind");
    axum::serve(listener, app).await.expect("serve");
}
