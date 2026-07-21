//! Sa-Token-Rs Axum 示例
//!
//! 演示 Sa-Token-Rs 在 axum 框架中的使用。

use axum::{
    routing::{get, post},
    Json, Router,
};
use sa_token::prelude::*;
use sa_token::stp::stp_interface::StpInterface;
use sa_token_axum::{CurrentLoginId, OptionalLoginId, SaTokenLayer};
use sa_token_jwt::{JwtConfig, SaJwtTemplate};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// 响应结构
#[derive(Serialize)]
struct ApiResponse {
    code: i32,
    msg: String,
    data: Option<serde_json::Value>,
}

impl ApiResponse {
    fn success(data: impl Serialize) -> Self {
        Self {
            code: 200,
            msg: "success".to_string(),
            data: Some(serde_json::to_value(data).unwrap_or_default()),
        }
    }

    fn error(msg: impl Into<String>) -> Self {
        Self {
            code: 500,
            msg: msg.into(),
            data: None,
        }
    }
}

/// 登录请求
#[derive(Deserialize)]
struct LoginRequest {
    username: String,
    password: String,
}

/// 登录响应
#[derive(Serialize)]
struct LoginResponse {
    token: String,
}

/// 权限数据源
struct MyStpInterface;

impl StpInterface for MyStpInterface {
    fn get_permission_list(&self, login_id: &str, _login_type: &str) -> Vec<String> {
        match login_id {
            "admin" => vec![
                "user:add".to_string(),
                "user:list".to_string(),
                "user:delete".to_string(),
                "system:config".to_string(),
            ],
            "user" => vec!["user:list".to_string()],
            _ => vec![],
        }
    }

    fn get_role_list(&self, login_id: &str, _login_type: &str) -> Vec<String> {
        match login_id {
            "admin" => vec!["admin".to_string(), "user".to_string()],
            "user" => vec!["user".to_string()],
            _ => vec![],
        }
    }
}

/// 初始化 Sa-Token
fn init_sa_token() {
    SaManager::set_config(Arc::new(SaTokenConfig {
        token_name: "satoken".to_string(),
        timeout: 60 * 60 * 24, // 1 天
        is_concurrent: true,
        is_share: true,
        ..Default::default()
    }));
    SaManager::set_sa_token_dao(Arc::new(SaTokenDaoMemory::new()));
    SaTokenContextMockUtil::set_mock_context();
    SaManager::set_stp_interface(Arc::new(MyStpInterface));
    SaManager::put_stp_logic(Arc::new(StpLogic::new("login")));
}

// ==================== Handlers ====================

/// 首页
async fn index() -> Json<ApiResponse> {
    Json(ApiResponse::success("Sa-Token-Rs Axum Demo"))
}

/// 登录
async fn login(Json(req): Json<LoginRequest>) -> Json<ApiResponse> {
    // 简单验证（实际项目中应查询数据库）
    if req.password != "123456" {
        return Json(ApiResponse::error("密码错误"));
    }

    // 登录
    match StpUtil::login(&req.username) {
        Ok(_) => {
            let token = StpUtil::get_token_value().unwrap_or_default();
            Json(ApiResponse::success(LoginResponse { token }))
        }
        Err(e) => Json(ApiResponse::error(e.to_string())),
    }
}

/// 登出
async fn logout(_login_id: CurrentLoginId) -> Json<ApiResponse> {
    match StpUtil::logout() {
        Ok(_) => Json(ApiResponse::success("登出成功")),
        Err(e) => Json(ApiResponse::error(e.to_string())),
    }
}

/// 获取用户信息
async fn user_info(login_id: CurrentLoginId) -> Json<ApiResponse> {
    Json(ApiResponse::success(serde_json::json!({
        "login_id": login_id.0,
        "is_login": true
    })))
}

/// 获取用户信息（可选登录）
async fn user_info_optional(login_id: OptionalLoginId) -> Json<ApiResponse> {
    match login_id.0 {
        Some(id) => Json(ApiResponse::success(serde_json::json!({
            "login_id": id,
            "is_login": true
        }))),
        None => Json(ApiResponse::success(serde_json::json!({
            "login_id": null,
            "is_login": false
        }))),
    }
}

/// 检查权限
async fn check_permission() -> Json<ApiResponse> {
    match StpUtil::has_permission("user:add") {
        Ok(true) => Json(ApiResponse::success(serde_json::json!({"has_permission": true}))),
        Ok(false) => Json(ApiResponse::success(serde_json::json!({"has_permission": false}))),
        Err(e) => Json(ApiResponse::error(e.to_string())),
    }
}

/// 检查角色
async fn check_role() -> Json<ApiResponse> {
    match StpUtil::has_role("admin") {
        Ok(true) => Json(ApiResponse::success(serde_json::json!({"has_role": true}))),
        Ok(false) => Json(ApiResponse::success(serde_json::json!({"has_role": false}))),
        Err(e) => Json(ApiResponse::error(e.to_string())),
    }
}

/// 获取 Token 信息
async fn token_info() -> Json<ApiResponse> {
    match StpUtil::get_token_info() {
        Ok(info) => Json(ApiResponse::success(serde_json::json!({
            "token_name": info.token_name,
            "token_value": info.token_value,
            "login_id": info.login_id,
            "token_timeout": info.token_timeout,
        }))),
        Err(e) => Json(ApiResponse::error(e.to_string())),
    }
}

/// JWT 登录
async fn jwt_login(Json(req): Json<LoginRequest>) -> Json<ApiResponse> {
    if req.password != "123456" {
        return Json(ApiResponse::error("密码错误"));
    }

    // 先登录
    if let Err(e) = StpUtil::login(&req.username) {
        return Json(ApiResponse::error(e.to_string()));
    }

    // 生成 JWT
    let config = JwtConfig::new("my-secret-key");
    let jwt = SaJwtTemplate::new(config);

    match jwt.create_token(&req.username, "login", 3600) {
        Ok(token) => Json(ApiResponse::success(serde_json::json!({
            "jwt_token": token
        }))),
        Err(e) => Json(ApiResponse::error(e.to_string())),
    }
}

#[tokio::main]
async fn main() {
    // 初始化 Sa-Token
    init_sa_token();

    // 创建路由
    let app = Router::new()
        .route("/", get(index))
        .route("/login", post(login))
        .route("/logout", post(logout))
        .route("/user/info", get(user_info))
        .route("/user/info/optional", get(user_info_optional))
        .route("/user/check-permission", get(check_permission))
        .route("/user/check-role", get(check_role))
        .route("/user/token-info", get(token_info))
        .route("/jwt/login", post(jwt_login))
        .layer(SaTokenLayer::new());

    // 启动服务器
    let addr = "0.0.0.0:8080";
    println!("🚀 Sa-Token-Rs Axum Demo 启动在 http://{}", addr);
    println!("📝 API 列表:");
    println!("   GET  /                    - 首页");
    println!("   POST /login               - 登录 (username, password)");
    println!("   POST /logout              - 登出");
    println!("   GET  /user/info           - 获取用户信息（需登录）");
    println!("   GET  /user/info/optional  - 获取用户信息（可选登录）");
    println!("   GET  /user/check-permission - 检查权限");
    println!("   GET  /user/check-role     - 检查角色");
    println!("   GET  /user/token-info     - 获取 Token 信息");
    println!("   POST /jwt/login           - JWT 登录");

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
