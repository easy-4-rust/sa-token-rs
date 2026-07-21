//! Sa-Token Axum 适配层
//!
//! 提供 Axum Web 框架的集成，包括：
//! - `SaTokenLayer` 中间件
//! - `CurrentLoginId` Extractor
//! - `AxumRequest` / `AxumResponse` / `AxumStorage` 上下文实现
//!
//! # 示例
//!
//! ```rust,ignore
//! use axum::{routing::get, Router, Json};
//! use sa_token_axum::{SaTokenLayer, CurrentLoginId};
//! use sa_token::prelude::*;
//!
//! async fn user_info(login_id: CurrentLoginId) -> Json<String> {
//!     Json(format!("Hello, {}!", login_id.0))
//! }
//!
//! let app = Router::new()
//!     .route("/user/info", get(user_info))
//!     .layer(SaTokenLayer::new());
//! ```

use std::any::Any;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use axum::extract::Request;
use axum::http::{header, StatusCode};
use axum::middleware::{self, Next};
use axum::response::{IntoResponse, Response};
use axum::Extension;

use sa_token_core::context::model::sa_cookie::SaCookie;
use sa_token_core::context::model::sa_request::SaRequest;
use sa_token_core::context::model::sa_response::SaResponse;
use sa_token_core::context::model::sa_storage::SaStorage;
use sa_token_core::context::sa_token_context::SaTokenContext;
use sa_token_core::context::sa_token_context_for_thread_local::SaTokenContextForThreadLocal;
use sa_token_core::manager::SaManager;
use sa_token_core::stp::stp_util::StpUtil;

// ==================== AxumRequest ====================

/// Axum 请求适配
pub struct AxumRequest {
    /// 请求路径
    path: String,
    /// 请求方法
    method: String,
    /// 请求头
    headers: HashMap<String, String>,
    /// 查询参数
    params: HashMap<String, String>,
    /// Cookie
    cookies: HashMap<String, String>,
}

impl AxumRequest {
    /// 从 axum Request 创建
    pub fn from_axum_request(req: &Request) -> Self {
        let path = req.uri().path().to_string();
        let method = req.method().to_string();

        let mut headers = HashMap::new();
        for (name, value) in req.headers() {
            if let Ok(v) = value.to_str() {
                headers.insert(name.to_string(), v.to_string());
            }
        }

        let mut params = HashMap::new();
        if let Some(query) = req.uri().query() {
            for pair in query.split('&') {
                if let Some((key, value)) = pair.split_once('=') {
                    params.insert(
                        urlencoding::decode(key).unwrap_or_default().to_string(),
                        urlencoding::decode(value).unwrap_or_default().to_string(),
                    );
                }
            }
        }

        let mut cookies = HashMap::new();
        if let Some(cookie_header) = headers.get("cookie") {
            for pair in cookie_header.split(';') {
                let pair = pair.trim();
                if let Some((key, value)) = pair.split_once('=') {
                    cookies.insert(key.trim().to_string(), value.trim().to_string());
                }
            }
        }

        Self {
            path,
            method,
            headers,
            params,
            cookies,
        }
    }
}

impl SaRequest for AxumRequest {
    fn source(&self) -> &dyn Any {
        self
    }

    fn get_param(&self, name: &str) -> Option<String> {
        self.params.get(name).cloned()
    }

    fn get_header(&self, name: &str) -> Option<String> {
        // 大小写不敏感
        let name_lower = name.to_lowercase();
        self.headers
            .iter()
            .find(|(k, _)| k.to_lowercase() == name_lower)
            .map(|(_, v)| v.clone())
    }

    fn get_cookie_value(&self, name: &str) -> Option<String> {
        self.cookies.get(name).cloned()
    }

    fn get_request_path(&self) -> String {
        self.path.clone()
    }

    fn get_url(&self) -> String {
        self.path.clone()
    }

    fn get_method(&self) -> String {
        self.method.clone()
    }

    fn get_host(&self) -> String {
        self.headers
            .get("host")
            .cloned()
            .unwrap_or_else(|| "localhost".to_string())
    }

    fn is_ajax(&self) -> bool {
        self.headers
            .get("x-requested-with")
            .map(|v| v.to_lowercase() == "xmlhttprequest")
            .unwrap_or(false)
    }

    fn forward(&self, _path: &str) {
        // axum 中转发需要在路由层处理
    }
}

// ==================== AxumResponse ====================

/// Axum 响应适配
pub struct AxumResponse {
    /// 响应头
    headers: RwLock<HashMap<String, String>>,
    /// Cookie 列表
    cookies: RwLock<Vec<SaCookie>>,
    /// 状态码
    status: RwLock<u16>,
}

impl Default for AxumResponse {
    fn default() -> Self {
        Self {
            headers: RwLock::new(HashMap::new()),
            cookies: RwLock::new(Vec::new()),
            status: RwLock::new(200),
        }
    }
}

impl SaResponse for AxumResponse {
    fn source(&self) -> &dyn Any {
        self
    }

    fn set_status(&self, sc: u16) {
        *self.status.write().unwrap() = sc;
    }

    fn set_header(&self, name: &str, value: &str) {
        self.headers
            .write()
            .unwrap()
            .insert(name.to_string(), value.to_string());
    }

    fn add_header(&self, name: &str, value: &str) {
        self.headers
            .write()
            .unwrap()
            .insert(name.to_string(), value.to_string());
    }

    fn add_cookie(&self, cookie: SaCookie) {
        self.cookies.write().unwrap().push(cookie);
    }

    fn delete_cookie(&self, name: &str) {
        self.cookies.write().unwrap().retain(|c| c.name != name);
    }

    fn redirect(&self, _url: &str) {
        // axum 中重定向需要返回 Redirect 响应
    }
}

// ==================== AxumStorage ====================

/// Axum 存储适配（请求级临时存储）
pub struct AxumStorage {
    /// 数据
    data: RwLock<HashMap<String, String>>,
}

impl Default for AxumStorage {
    fn default() -> Self {
        Self {
            data: RwLock::new(HashMap::new()),
        }
    }
}

impl SaStorage for AxumStorage {
    fn source(&self) -> &dyn Any {
        self
    }

    fn get(&self, key: &str) -> Option<String> {
        self.data.read().unwrap().get(key).cloned()
    }

    fn set(&self, key: &str, value: &str) {
        self.data
            .write()
            .unwrap()
            .insert(key.to_string(), value.to_string());
    }

    fn delete(&self, key: &str) {
        self.data.write().unwrap().remove(key);
    }
}

// ==================== AxumContext ====================

/// Axum 上下文实现
pub struct AxumContext {
    /// 请求
    request: Arc<AxumRequest>,
    /// 响应
    response: Arc<AxumResponse>,
    /// 存储
    storage: Arc<AxumStorage>,
}

impl AxumContext {
    /// 创建新的 Axum 上下文
    pub fn new(request: AxumRequest) -> Self {
        Self {
            request: Arc::new(request),
            response: Arc::new(AxumResponse::default()),
            storage: Arc::new(AxumStorage::default()),
        }
    }
}

impl SaTokenContext for AxumContext {
    fn set_context(
        &self,
        _req: Arc<dyn SaRequest>,
        _res: Arc<dyn SaResponse>,
        _stg: Arc<dyn SaStorage>,
    ) {
        // Axum 上下文在创建时已设置
    }

    fn clear_context(&self) {
        // 清理存储
        self.storage.data.write().unwrap().clear();
    }

    fn is_valid(&self) -> bool {
        true
    }

    fn request(&self) -> Arc<dyn SaRequest> {
        self.request.clone()
    }

    fn response(&self) -> Arc<dyn SaResponse> {
        self.response.clone()
    }

    fn storage(&self) -> Arc<dyn SaStorage> {
        self.storage.clone()
    }
}

// ==================== SaTokenLayer ====================

/// Sa-Token Axum 中间件层
///
/// 自动将请求注入到 Sa-Token 上下文中。
#[derive(Clone)]
pub struct SaTokenLayer;

impl SaTokenLayer {
    /// 创建新的 SaTokenLayer
    pub fn new() -> Self {
        Self
    }
}

impl Default for SaTokenLayer {
    fn default() -> Self {
        Self::new()
    }
}

impl<S> tower::Layer<S> for SaTokenLayer {
    type Service = SaTokenService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        SaTokenService { inner }
    }
}

/// Sa-Token 中间件服务
#[derive(Clone)]
pub struct SaTokenService<S> {
    inner: S,
}

impl<S> tower::Service<Request> for SaTokenService<S>
where
    S: tower::Service<Request, Response = Response> + Send + Clone + 'static,
    S::Future: Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = futures_util::future::BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request) -> Self::Future {
        let mut inner = self.inner.clone();

        Box::pin(async move {
            // 创建 Axum 上下文
            let axum_req = AxumRequest::from_axum_request(&req);
            let ctx = AxumContext::new(axum_req);

            // 设置到 Sa-Token 上下文
            let ctx_arc = Arc::new(ctx);
            SaManager::set_sa_token_context(ctx_arc.clone());

            // 调用下一个中间件
            let response = inner.call(req).await?;

            Ok(response)
        })
    }
}

// ==================== Extractors ====================

/// 当前登录 ID Extractor
///
/// 从请求中提取当前登录的用户 ID。如果未登录，返回 401 错误。
///
/// # 示例
///
/// ```rust,ignore
/// use sa_token_axum::CurrentLoginId;
///
/// async fn handler(login_id: CurrentLoginId) -> String {
///     format!("Hello, {}!", login_id.0)
/// }
/// ```
pub struct CurrentLoginId(pub String);

#[async_trait::async_trait]
impl<S: Send + Sync> axum::extract::FromRequestParts<S> for CurrentLoginId {
    type Rejection = (StatusCode, String);

    async fn from_request_parts(
        _parts: &mut axum::http::request::Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        match StpUtil::get_login_id() {
            Ok(id) => Ok(CurrentLoginId(id)),
            Err(e) => Err((StatusCode::UNAUTHORIZED, e.to_string())),
        }
    }
}

/// 可选登录 ID Extractor
///
/// 从请求中提取当前登录的用户 ID。如果未登录，返回 None。
///
/// # 示例
///
/// ```rust,ignore
/// use sa_token_axum::OptionalLoginId;
///
/// async fn handler(login_id: OptionalLoginId) -> String {
///     match login_id.0 {
///         Some(id) => format!("Hello, {}!", id),
///         None => "Hello, Guest!".to_string(),
///     }
/// }
/// ```
pub struct OptionalLoginId(pub Option<String>);

#[async_trait::async_trait]
impl<S: Send + Sync> axum::extract::FromRequestParts<S> for OptionalLoginId {
    type Rejection = (StatusCode, String);

    async fn from_request_parts(
        _parts: &mut axum::http::request::Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        Ok(OptionalLoginId(StpUtil::get_login_id_default_null()))
    }
}

/// 权限检查 Extractor
///
/// 检查当前用户是否具有指定权限。如果不满足，返回 403 错误。
///
/// # 示例
///
/// ```rust,ignore
/// use sa_token_axum::RequirePermission;
///
/// async fn handler(_: RequirePermission) -> String {
///     "You have permission!".to_string()
/// }
///
/// // 路由中使用
/// let app = Router::new()
///     .route("/admin", get(handler))
///     .layer(RequirePermission::layer("user:add"));
/// ```
pub struct RequirePermission {
    /// 需要的权限
    permission: String,
}

impl RequirePermission {
    /// 创建权限检查层
    pub fn layer(permission: impl Into<String>) -> RequirePermissionLayer {
        RequirePermissionLayer {
            permission: permission.into(),
        }
    }
}

/// 权限检查层
pub struct RequirePermissionLayer {
    permission: String,
}

impl<S> tower::Layer<S> for RequirePermissionLayer {
    type Service = RequirePermissionService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        RequirePermissionService {
            inner,
            permission: self.permission.clone(),
        }
    }
}

/// 权限检查服务
#[derive(Clone)]
pub struct RequirePermissionService<S> {
    inner: S,
    permission: String,
}

impl<S> tower::Service<Request> for RequirePermissionService<S>
where
    S: tower::Service<Request, Response = Response> + Send + Clone + 'static,
    S::Future: Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = futures_util::future::BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request) -> Self::Future {
        let mut inner = self.inner.clone();
        let permission = self.permission.clone();

        Box::pin(async move {
            match StpUtil::check_permission(&permission) {
                Ok(_) => inner.call(req).await,
                Err(e) => Ok((StatusCode::FORBIDDEN, e.to_string()).into_response()),
            }
        })
    }
}

/// 角色检查 Extractor
///
/// 检查当前用户是否具有指定角色。如果不满足，返回 403 错误。
///
/// # 示例
///
/// ```rust,ignore
/// use sa_token_axum::RequireRole;
///
/// async fn handler(_: RequireRole) -> String {
///     "You are admin!".to_string()
/// }
///
/// // 路由中使用
/// let app = Router::new()
///     .route("/admin", get(handler))
///     .layer(RequireRole::layer("admin"));
/// ```
pub struct RequireRole {
    /// 需要的角色
    role: String,
}

impl RequireRole {
    /// 创建角色检查层
    pub fn layer(role: impl Into<String>) -> RequireRoleLayer {
        RequireRoleLayer { role: role.into() }
    }
}

/// 角色检查层
pub struct RequireRoleLayer {
    role: String,
}

impl<S> tower::Layer<S> for RequireRoleLayer {
    type Service = RequireRoleService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        RequireRoleService {
            inner,
            role: self.role.clone(),
        }
    }
}

/// 角色检查服务
#[derive(Clone)]
pub struct RequireRoleService<S> {
    inner: S,
    role: String,
}

impl<S> tower::Service<Request> for RequireRoleService<S>
where
    S: tower::Service<Request, Response = Response> + Send + Clone + 'static,
    S::Future: Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = futures_util::future::BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request) -> Self::Future {
        let mut inner = self.inner.clone();
        let role = self.role.clone();

        Box::pin(async move {
            match StpUtil::check_role(&role) {
                Ok(_) => inner.call(req).await,
                Err(e) => Ok((StatusCode::FORBIDDEN, e.to_string()).into_response()),
            }
        })
    }
}
