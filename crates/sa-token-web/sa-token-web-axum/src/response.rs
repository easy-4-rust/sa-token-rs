//! Axum HTTP response adapter (`SaResponse`).

use std::any::Any;
use std::collections::HashMap;
use std::sync::RwLock;

use sa_token_core::context::model::sa_cookie::SaCookie;
use sa_token_core::context::model::sa_response::SaResponse;

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
