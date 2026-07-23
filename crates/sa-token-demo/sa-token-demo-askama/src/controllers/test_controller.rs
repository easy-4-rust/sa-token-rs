//! 测试控制器（对应 Java `com.pj.test.TestController`）。

use std::sync::Arc;

use askama::Template;
use axum::extract::{Query, State};
use axum::response::{Html, IntoResponse, Json};
use sa_token::prelude::*;
use sa_token_askama::SaTokenDialect;
use serde::Deserialize;
use serde_json::json;

use crate::util::AjaxJson;
use crate::views::index_template::IndexTemplate;

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

/// 首页：渲染 Askama 模板（对应 Java `index()` → ModelAndView("index.html")）。
pub async fn index(State(dialect): State<Arc<SaTokenDialect>>) -> impl IntoResponse {
    let is_login = dialect.is_login();
    let mut session_name = String::new();
    if is_login {
        if let Ok(session) = StpUtil::get_session() {
            if let Some(name) = session.get("name").and_then(|v| v.as_str()) {
                session_name = name.to_string();
            }
        }
    }

    // 预计算 sa:* 标签断言（Askama 无运行时自定义属性处理器）
    let tpl = IndexTemplate {
        is_login,
        session_name,
        sa_login: dialect.evaluate("login", None),
        sa_not_login: dialect.evaluate("notLogin", None),
        sa_has_role_admin: dialect.evaluate("hasRole", Some("admin")),
        sa_has_role_and: dialect.evaluate("hasRoleAnd", Some("admin, ceo, cto")),
        sa_has_role_or: dialect.evaluate("hasRoleOr", Some("admin, ceo, cto")),
        sa_not_role_admin: dialect.evaluate("notRole", Some("admin")),
        sa_has_permission_user_add: dialect.evaluate("hasPermission", Some("user-add")),
        sa_has_permission_and: dialect
            .evaluate("hasPermissionAnd", Some("user-add, user-delete, user-get")),
        sa_has_permission_or: dialect
            .evaluate("hasPermissionOr", Some("user-add, user-delete, user-get")),
        sa_not_permission_user_add: dialect.evaluate("notPermission", Some("user-add")),
    };

    match tpl.render() {
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
