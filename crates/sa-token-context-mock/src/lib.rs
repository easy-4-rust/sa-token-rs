//! Mock 上下文（对应 Java `cn.dev33.satoken.context.mock`）。
//!
//! 提供 Mock 上下文实现，用于测试和非 Web 场景。

use std::any::Any;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use sa_token_core::context::model::sa_cookie::SaCookie;
use sa_token_core::context::model::sa_request::SaRequest;
use sa_token_core::context::model::sa_response::SaResponse;
use sa_token_core::context::model::sa_storage::SaStorage;
use sa_token_core::context::sa_token_context::SaTokenContext;
use sa_token_core::context::sa_token_context_for_thread_local::SaTokenContextForThreadLocal;
use sa_token_core::manager::SaManager;

/// Mock 请求
pub struct SaRequestForMock {
    /// 参数
    params: RwLock<HashMap<String, String>>,
    /// 请求头
    headers: RwLock<HashMap<String, String>>,
    /// Cookie
    cookies: RwLock<HashMap<String, String>>,
    /// 请求路径
    path: String,
}

impl Default for SaRequestForMock {
    fn default() -> Self {
        Self {
            params: RwLock::new(HashMap::new()),
            headers: RwLock::new(HashMap::new()),
            cookies: RwLock::new(HashMap::new()),
            path: "/".to_string(),
        }
    }
}

impl SaRequestForMock {
    /// 设置参数
    pub fn set_param(&self, name: impl Into<String>, value: impl Into<String>) {
        self.params.write().unwrap().insert(name.into(), value.into());
    }

    /// 设置请求头
    pub fn set_header(&self, name: impl Into<String>, value: impl Into<String>) {
        self.headers.write().unwrap().insert(name.into(), value.into());
    }

    /// 设置 Cookie
    pub fn set_cookie(&self, name: impl Into<String>, value: impl Into<String>) {
        self.cookies.write().unwrap().insert(name.into(), value.into());
    }
}

impl SaRequest for SaRequestForMock {
    fn source(&self) -> &dyn Any {
        self
    }

    fn get_param(&self, name: &str) -> Option<String> {
        self.params.read().unwrap().get(name).cloned()
    }

    fn get_header(&self, name: &str) -> Option<String> {
        self.headers.read().unwrap().get(name).cloned()
    }

    fn get_cookie_value(&self, name: &str) -> Option<String> {
        self.cookies.read().unwrap().get(name).cloned()
    }

    fn get_request_path(&self) -> String {
        self.path.clone()
    }

    fn get_url(&self) -> String {
        format!("http://localhost{}", self.path)
    }

    fn get_method(&self) -> String {
        "GET".to_string()
    }

    fn get_host(&self) -> String {
        "localhost".to_string()
    }

    fn is_ajax(&self) -> bool {
        false
    }

    fn forward(&self, _path: &str) {}
}

/// Mock 响应
pub struct SaResponseForMock {
    /// 响应头
    headers: RwLock<HashMap<String, String>>,
    /// Cookie
    cookies: RwLock<Vec<SaCookie>>,
    /// 状态码
    status: RwLock<u16>,
}

impl Default for SaResponseForMock {
    fn default() -> Self {
        Self {
            headers: RwLock::new(HashMap::new()),
            cookies: RwLock::new(Vec::new()),
            status: RwLock::new(200),
        }
    }
}

impl SaResponse for SaResponseForMock {
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

    fn redirect(&self, _url: &str) {}
}

/// Mock 存储
pub struct SaStorageForMock {
    /// 数据
    data: RwLock<HashMap<String, String>>,
}

impl Default for SaStorageForMock {
    fn default() -> Self {
        Self {
            data: RwLock::new(HashMap::new()),
        }
    }
}

impl SaStorage for SaStorageForMock {
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

/// Mock 上下文工具
pub struct SaTokenContextMockUtil;

impl SaTokenContextMockUtil {
    /// 设置 Mock 上下文
    pub fn set_mock_context() {
        let ctx = SaTokenContextForThreadLocal;
        let req = Arc::new(SaRequestForMock::default());
        let res = Arc::new(SaResponseForMock::default());
        let stg = Arc::new(SaStorageForMock::default());
        ctx.set_context(req, res, stg);
        SaManager::set_sa_token_context(Arc::new(ctx));
    }

    /// 清除 Mock 上下文
    pub fn clear_context() {
        let ctx = SaManager::sa_token_context();
        ctx.clear_context();
    }

    /// 获取 Mock 请求（用于设置参数等）
    pub fn mock_request() -> Arc<SaRequestForMock> {
        let req = Arc::new(SaRequestForMock::default());
        let res = Arc::new(SaResponseForMock::default());
        let stg = Arc::new(SaStorageForMock::default());
        let ctx = SaTokenContextForThreadLocal;
        ctx.set_context(req.clone(), res, stg);
        SaManager::set_sa_token_context(Arc::new(ctx));
        req
    }
}
