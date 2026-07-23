//! 测试控制器（对应 Java `com.pj.test.TestController`）。

use std::sync::Arc;

use axum::extract::{Query, State};
use axum::response::{Html, IntoResponse, Json};
use sa_token::prelude::*;
use serde::Deserialize;
use serde_json::json;
use tera::{Context, Tera};

use crate::util::AjaxJson;

/// 登录查询参数（对应 Java `@RequestParam(defaultValue="10001")`）。
#[derive(Debug, Deserialize)]
pub struct LoginQuery {
    /// 登录账号 id
    #[serde(default = "default_id")]
    pub id: String,
}

fn default_id() -> String {
    "10001".into()
}

/// 首页：渲染 Tera 模板（对应 Java `index()` → ModelAndView("index")）。
pub async fn index(State(tera): State<Arc<Tera>>) -> impl IntoResponse {
    let mut ctx = Context::new();
    let is_login = StpUtil::is_login().unwrap_or(false);
    ctx.insert("is_login", &is_login);
    if is_login {
        if let Ok(session) = StpUtil::get_session() {
            if let Some(name) = session.get("name").and_then(|v| v.as_str()) {
                ctx.insert("session_name", &name.to_string());
            }
        }
    }
    match tera.render("index.html", &ctx) {
        Ok(html) => Html(html).into_response(),
        Err(e) => (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            format!("模板渲染失败: {e}"),
        )
            .into_response(),
    }
}

/// 登录（对应 Java `login`）。
pub async fn login(Query(q): Query<LoginQuery>) -> Json<AjaxJson> {
    match StpUtil::login(&q.id) {
        Ok(()) => {
            if let Ok(mut session) = StpUtil::get_session() {
                session.set("name", json!("zhangsan"));
            }
            Json(AjaxJson::get_success())
        }
        Err(e) => Json(AjaxJson::get_error(e.to_string())),
    }
}

/// 注销（对应 Java `logout`）。
pub async fn logout() -> Json<AjaxJson> {
    match StpUtil::logout() {
        Ok(()) => Json(AjaxJson::get_success()),
        Err(e) => Json(AjaxJson::get_error(e.to_string())),
    }
}
