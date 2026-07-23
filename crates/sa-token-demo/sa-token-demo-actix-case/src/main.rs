//! Sa-Token-Rs Actix Case Demo
//!
//! 框架映射：Quarkus → actix-web；对应 Java `sa-token-demo-case`

mod util;

use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use actix_web::{App, HttpRequest, HttpResponse, HttpServer, web};
use sa_token::prelude::{
    AsyncSaTokenRuntime, AsyncStpUtil, SaLoginParameter, SaTokenConfig, SaTokenDaoMemory,
};
use sa_token_core::context::mock::sa_request_for_mock::SaRequestForMock;
use sa_token_core::context::mock::sa_response_for_mock::SaResponseForMock;
use sa_token_core::context::mock::sa_storage_for_mock::SaStorageForMock;
use sa_token_core::context::model::sa_request::SaRequest;
use sa_token_core::context::model::sa_response::SaResponse;
use sa_token_core::context::model::sa_storage::SaStorage;
use sa_token_core::context::model::sa_token_context_model_box::SaTokenContextModelBox;
use sa_token_core::context::sa_token_context::SaTokenContext;
use sa_token_core::stp::StpInterface;
use serde::Deserialize;
use serde_json::json;

use crate::util::AjaxJson;

/// 演示用请求上下文。
struct DemoContext {
    request: Arc<dyn SaRequest>,
    response: Arc<dyn SaResponse>,
    storage: Arc<dyn SaStorage>,
}

impl DemoContext {
    /// 创建空 Mock 上下文。
    fn new() -> Self {
        Self {
            request: Arc::new(SaRequestForMock::default()),
            response: Arc::new(SaResponseForMock::new()),
            storage: Arc::new(SaStorageForMock::new()),
        }
    }
}

impl SaTokenContext for DemoContext {
    fn set_context(
        &self,
        _request: Arc<dyn SaRequest>,
        _response: Arc<dyn SaResponse>,
        _storage: Arc<dyn SaStorage>,
    ) {
    }

    fn clear_context(&self) {
        self.storage.delete("satoken");
    }

    fn is_valid(&self) -> bool {
        true
    }

    fn model_box(&self) -> SaTokenContextModelBox {
        SaTokenContextModelBox::new(
            Arc::clone(&self.request),
            Arc::clone(&self.response),
            Arc::clone(&self.storage),
        )
    }

    fn request(&self) -> Arc<dyn SaRequest> {
        Arc::clone(&self.request)
    }

    fn response(&self) -> Arc<dyn SaResponse> {
        Arc::clone(&self.response)
    }

    fn storage(&self) -> Arc<dyn SaStorage> {
        Arc::clone(&self.storage)
    }
}

/// 权限 / 角色数据源。
struct StpInterfaceImpl;

impl StpInterface for StpInterfaceImpl {
    fn get_permission_list(&self, _login_id: &str, _login_type: &str) -> Vec<String> {
        vec!["user-add".into(), "user-delete".into(), "101".into()]
    }

    fn get_role_list(&self, _login_id: &str, _login_type: &str) -> Vec<String> {
        vec!["admin".into(), "super-admin".into()]
    }
}

/// 账号密码登录参数。
#[derive(Debug, Deserialize)]
struct DoLoginQuery {
    #[serde(default)]
    name: String,
    #[serde(default)]
    pwd: String,
}

/// 登录 id 参数。
#[derive(Debug, Deserialize)]
struct IdQuery {
    #[serde(default = "default_id")]
    id: String,
}

fn default_id() -> String {
    "10001".into()
}

/// 封禁参数。
#[derive(Debug, Deserialize)]
struct DisableQuery {
    #[serde(default = "default_id")]
    id: String,
    #[serde(default = "default_disable_time")]
    time: i64,
}

fn default_disable_time() -> i64 {
    86_400
}

/// 二级认证参数。
#[derive(Debug, Deserialize)]
struct SafeQuery {
    #[serde(default = "default_safe_time")]
    safe_time: i64,
}

fn default_safe_time() -> i64 {
    120
}

/// 临时 Token 参数。
#[derive(Debug, Deserialize)]
struct TempTokenQuery {
    #[serde(default)]
    value: String,
    #[serde(default = "default_temp_timeout")]
    timeout: i64,
}

fn default_temp_timeout() -> i64 {
    60
}

/// 从请求提取 satoken。
fn token_from_req(req: &HttpRequest) -> Option<String> {
    req.headers()
        .get("satoken")
        .and_then(|v| v.to_str().ok())
        .map(str::to_string)
        .or_else(|| {
            req.headers()
                .get(actix_web::http::header::AUTHORIZATION)
                .and_then(|v| v.to_str().ok())
                .and_then(|v| v.strip_prefix("Bearer "))
                .map(str::to_string)
        })
}

/// 绑定 token 到 DemoContext。
fn bind_token(util: &AsyncStpUtil, token: &str) {
    let name = util.logic().runtime().config().token_name.clone();
    util.logic().runtime().context().storage().set(&name, token);
}

/// 要求已登录。
async fn require_login(util: &AsyncStpUtil, req: &HttpRequest) -> Result<(String, String), String> {
    let token = token_from_req(req).ok_or_else(|| "未登录".to_string())?;
    let login_id = util
        .get_login_id_by_token(&token)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "token 无效".to_string())?;
    bind_token(util, &token);
    Ok((token, login_id))
}

/// 构建 AsyncStpUtil。
fn build_util() -> AsyncStpUtil {
    let runtime = AsyncSaTokenRuntime::new(
        Arc::new(SaTokenConfig {
            token_name: "satoken".into(),
            timeout: 2_592_000,
            is_concurrent: true,
            is_share: false,
            is_log: true,
            ..Default::default()
        }),
        Arc::new(SaTokenDaoMemory::new()),
        Arc::new(DemoContext::new()),
    )
    .with_stp_interface(Arc::new(StpInterfaceImpl));

    AsyncStpUtil::new("login", Arc::new(runtime))
}

/// 登录 —— `/acc/doLogin`
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

/// 是否登录 —— `/acc/isLogin`
async fn is_login(util: web::Data<AsyncStpUtil>, req: HttpRequest) -> HttpResponse {
    let logged_in = match token_from_req(&req) {
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

/// 注销 —— `/acc/logout`
async fn logout(util: web::Data<AsyncStpUtil>, req: HttpRequest) -> HttpResponse {
    match require_login(&util, &req).await {
        Ok((_token, login_id)) => match util.logout_by_login_id(&login_id).await {
            Ok(()) => HttpResponse::Ok().json(AjaxJson::ok()),
            Err(e) => HttpResponse::Ok().json(AjaxJson::error(e.to_string())),
        },
        Err(e) => HttpResponse::Ok().json(AjaxJson::error(e)),
    }
}

/// Token 信息 —— `/acc/tokenInfo`
async fn token_info(util: web::Data<AsyncStpUtil>, req: HttpRequest) -> HttpResponse {
    match require_login(&util, &req).await {
        Ok((token, login_id)) => HttpResponse::Ok().json(AjaxJson::ok_data(json!({
            "token_value": token,
            "login_id": login_id,
            "token_name": util.logic().runtime().config().token_name,
        }))),
        Err(e) => HttpResponse::Ok().json(AjaxJson::error(e)),
    }
}

/// 测试登录 —— `/test/login`
async fn test_login(util: web::Data<AsyncStpUtil>, query: web::Query<IdQuery>) -> HttpResponse {
    match util.login(&query.id).await {
        Ok(token) => HttpResponse::Ok().json(AjaxJson::ok().set_data(json!({ "token": token }))),
        Err(e) => HttpResponse::Ok().json(AjaxJson::error(e.to_string())),
    }
}

/// 测试注销 —— `/test/logout`
async fn test_logout(util: web::Data<AsyncStpUtil>, req: HttpRequest) -> HttpResponse {
    match require_login(&util, &req).await {
        Ok((_token, login_id)) => match util.logout_by_login_id(&login_id).await {
            Ok(()) => HttpResponse::Ok().json(AjaxJson::ok()),
            Err(e) => HttpResponse::Ok().json(AjaxJson::error(e.to_string())),
        },
        Err(e) => HttpResponse::Ok().json(AjaxJson::error(e)),
    }
}

/// 角色校验 —— `/test/testRole`
async fn test_role(util: web::Data<AsyncStpUtil>, req: HttpRequest) -> HttpResponse {
    if let Err(e) = require_login(&util, &req).await {
        return HttpResponse::Ok().json(AjaxJson::error(e));
    }
    match util.check_role("admin").await {
        Ok(()) => HttpResponse::Ok().json(AjaxJson::ok_msg("角色测试通过")),
        Err(e) => HttpResponse::Ok().json(AjaxJson::error(e.to_string())),
    }
}

/// 权限校验 —— `/test/testJur`
async fn test_jur(util: web::Data<AsyncStpUtil>, req: HttpRequest) -> HttpResponse {
    if let Err(e) = require_login(&util, &req).await {
        return HttpResponse::Ok().json(AjaxJson::error(e));
    }
    match util.check_permission("user-add").await {
        Ok(()) => HttpResponse::Ok().json(AjaxJson::ok_msg("权限测试通过")),
        Err(e) => HttpResponse::Ok().json(AjaxJson::error(e.to_string())),
    }
}

/// Account-Session —— `/test/session`
async fn test_session(util: web::Data<AsyncStpUtil>, req: HttpRequest) -> HttpResponse {
    let (_token, login_id) = match require_login(&util, &req).await {
        Ok(v) => v,
        Err(e) => return HttpResponse::Ok().json(AjaxJson::error(e)),
    };
    match util.get_session_by_login_id(&login_id).await {
        Ok(mut session) => {
            let prev = session.get("name").cloned();
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_secs() as i64)
                .unwrap_or(0);
            session.set("name", json!(now));
            if let Err(e) = util.logic().runtime().dao().update_session(&session).await {
                return HttpResponse::Ok().json(AjaxJson::error(e.to_string()));
            }
            HttpResponse::Ok().json(AjaxJson::ok_data(json!({
                "session_id": session.id(),
                "prev_name": prev,
                "name": session.get("name"),
            })))
        }
        Err(e) => HttpResponse::Ok().json(AjaxJson::error(e.to_string())),
    }
}

/// 踢人下线 —— `/test/kickOut`
async fn kick_out(util: web::Data<AsyncStpUtil>, query: web::Query<IdQuery>) -> HttpResponse {
    match util.kickout_by_login_id(&query.id).await {
        Ok(()) => HttpResponse::Ok().json(AjaxJson::ok_msg(format!("已踢下线：{}", query.id))),
        Err(e) => HttpResponse::Ok().json(AjaxJson::error(e.to_string())),
    }
}

/// 开启二级认证 —— `/test/openSafe`
async fn open_safe(
    util: web::Data<AsyncStpUtil>,
    req: HttpRequest,
    query: web::Query<SafeQuery>,
) -> HttpResponse {
    if let Err(e) = require_login(&util, &req).await {
        return HttpResponse::Ok().json(AjaxJson::error(e));
    }
    match util.open_safe(query.safe_time).await {
        Ok(()) => HttpResponse::Ok().json(AjaxJson::ok_msg(format!(
            "已开启二级认证，有效期 {} 秒",
            query.safe_time
        ))),
        Err(e) => HttpResponse::Ok().json(AjaxJson::error(e.to_string())),
    }
}

/// 校验二级认证 —— `/test/checkSafe`
async fn check_safe(util: web::Data<AsyncStpUtil>, req: HttpRequest) -> HttpResponse {
    if let Err(e) = require_login(&util, &req).await {
        return HttpResponse::Ok().json(AjaxJson::error(e));
    }
    match util.logic().check_safe().await {
        Ok(()) => {
            let is_safe = util.is_safe().await.unwrap_or(false);
            HttpResponse::Ok().json(AjaxJson::ok_msg(format!("二级认证通过，is_safe={is_safe}")))
        }
        Err(e) => HttpResponse::Ok().json(AjaxJson::error(e.to_string())),
    }
}

/// 封禁账号 —— `/test/disable`
async fn disable(util: web::Data<AsyncStpUtil>, query: web::Query<DisableQuery>) -> HttpResponse {
    match util.disable(&query.id, query.time).await {
        Ok(()) => HttpResponse::Ok().json(AjaxJson::ok_msg(format!(
            "已封禁 {}，时长 {} 秒",
            query.id, query.time
        ))),
        Err(e) => HttpResponse::Ok().json(AjaxJson::error(e.to_string())),
    }
}

/// 是否封禁 —— `/test/isDisable`
async fn is_disable(util: web::Data<AsyncStpUtil>, query: web::Query<IdQuery>) -> HttpResponse {
    match util.is_disable(&query.id).await {
        Ok(disabled) => HttpResponse::Ok().json(AjaxJson::ok_msg(format!(
            "账号 {} 是否封禁：{disabled}",
            query.id
        ))),
        Err(e) => HttpResponse::Ok().json(AjaxJson::error(e.to_string())),
    }
}

/// 身份切换 —— `/test/switchTo`
async fn switch_to(
    util: web::Data<AsyncStpUtil>,
    req: HttpRequest,
    query: web::Query<IdQuery>,
) -> HttpResponse {
    if let Err(e) = require_login(&util, &req).await {
        return HttpResponse::Ok().json(AjaxJson::error(e));
    }
    match util.switch_to(&query.id).await {
        Ok(()) => HttpResponse::Ok().json(AjaxJson::ok_msg(format!(
            "已切换到 {}，is_switch={}",
            query.id,
            util.is_switch()
        ))),
        Err(e) => HttpResponse::Ok().json(AjaxJson::error(e.to_string())),
    }
}

/// 结束身份切换 —— `/test/endSwitch`
async fn end_switch(util: web::Data<AsyncStpUtil>, req: HttpRequest) -> HttpResponse {
    if let Err(e) = require_login(&util, &req).await {
        return HttpResponse::Ok().json(AjaxJson::error(e));
    }
    util.end_switch();
    HttpResponse::Ok().json(AjaxJson::ok_msg(format!(
        "已结束切换，is_switch={}",
        util.is_switch()
    )))
}

/// 临时 Token —— `/test/tempToken`
async fn temp_token(
    util: web::Data<AsyncStpUtil>,
    query: web::Query<TempTokenQuery>,
) -> HttpResponse {
    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);
    let token = format!("temp_{millis:x}");
    let key = format!("satoken:temp:{token}");
    let value = if query.value.is_empty() {
        "demo-temp-value".to_string()
    } else {
        query.value.clone()
    };
    match util
        .logic()
        .runtime()
        .dao()
        .set(&key, &value, query.timeout)
        .await
    {
        Ok(()) => HttpResponse::Ok().json(AjaxJson::ok_data(json!({
            "temp_token": token,
            "value": value,
            "timeout": query.timeout,
            "note": "Async DAO demo; SaTempUtil needs SaManager sync init",
        }))),
        Err(e) => HttpResponse::Ok().json(AjaxJson::error(e.to_string())),
    }
}

/// lasting cookie 登录 —— `/test/loginByParam`
async fn login_by_param(util: web::Data<AsyncStpUtil>, query: web::Query<IdQuery>) -> HttpResponse {
    let param = SaLoginParameter::create()
        .set_is_lasting_cookie(true)
        .set_timeout(2_592_000);
    match util.login_with_param(&query.id, &param).await {
        Ok(token) => HttpResponse::Ok().json(AjaxJson::ok().set_data(json!({
            "token": token,
            "is_lasting_cookie": true,
            "timeout": 2_592_000,
        }))),
        Err(e) => HttpResponse::Ok().json(AjaxJson::error(e.to_string())),
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let util = build_util();
    let addr = ("0.0.0.0", 8102);

    println!("🚀 Sa-Token-Rs Actix Case Demo（对应 sa-token-demo-case）");
    println!("   http://{}:{}", addr.0, addr.1);
    println!("   Header: satoken=<token>");

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(util.clone()))
            .route("/acc/doLogin", web::get().to(do_login))
            .route("/acc/doLogin", web::post().to(do_login))
            .route("/acc/isLogin", web::get().to(is_login))
            .route("/acc/logout", web::get().to(logout))
            .route("/acc/tokenInfo", web::get().to(token_info))
            .route("/test/login", web::get().to(test_login))
            .route("/test/logout", web::get().to(test_logout))
            .route("/test/testRole", web::get().to(test_role))
            .route("/test/testJur", web::get().to(test_jur))
            .route("/test/session", web::get().to(test_session))
            .route("/test/kickOut", web::get().to(kick_out))
            .route("/test/openSafe", web::get().to(open_safe))
            .route("/test/checkSafe", web::get().to(check_safe))
            .route("/test/disable", web::get().to(disable))
            .route("/test/isDisable", web::get().to(is_disable))
            .route("/test/switchTo", web::get().to(switch_to))
            .route("/test/endSwitch", web::get().to(end_switch))
            .route("/test/tempToken", web::get().to(temp_token))
            .route("/test/loginByParam", web::get().to(login_by_param))
    })
    .bind(addr)?
    .run()
    .await
}
