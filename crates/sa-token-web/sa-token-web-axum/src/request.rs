//! Axum HTTP request adapter (`SaRequest`).

use std::any::Any;
use std::collections::HashMap;

use axum::extract::Request;
use sa_token_core::context::model::sa_request::SaRequest;

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

    /// Returns normalized headers for token extraction helpers.
    pub fn header_pairs(&self) -> Vec<(String, String)> {
        self.headers.iter().map(|(k, v)| (k.clone(), v.clone())).collect()
    }

    /// Returns normalized cookies for token extraction helpers.
    pub fn cookie_pairs(&self) -> Vec<(String, String)> {
        self.cookies.iter().map(|(k, v)| (k.clone(), v.clone())).collect()
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
