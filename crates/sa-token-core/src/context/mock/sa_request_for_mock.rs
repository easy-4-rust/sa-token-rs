//! `SaRequestForMock` —— 1:1 对应 Java `cn.dev33.satoken.context.model.SaRequestForMock`
//!
//! 用于在非 Web 环境（CLI / 测试）中模拟一个 `SaRequest`。

use std::collections::HashMap;

use crate::context::model::sa_cookie::SaCookie;
use crate::context::model::sa_request::SaRequest;

/// `SaRequest` 的 Mock 实现
#[derive(Debug, Default)]
pub struct SaRequestForMock {
    /// 请求方法
    pub method: String,
    /// 请求 URL 路径
    pub url: String,
    /// 请求参数
    pub query: HashMap<String, String>,
    /// Header
    pub headers: HashMap<String, String>,
    /// Body（原始字符串）
    pub body: String,
    /// 客户端 IP
    pub remote_addr: String,
    /// 主机名
    pub host: String,
}

impl SaRequestForMock {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_url(mut self, url: impl Into<String>) -> Self {
        self.url = url.into();
        self
    }

    pub fn with_method(mut self, method: impl Into<String>) -> Self {
        self.method = method.into();
        self
    }

    pub fn with_query(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.query.insert(key.into(), value.into());
        self
    }

    pub fn with_header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.insert(key.into(), value.into());
        self
    }

    pub fn with_body(mut self, body: impl Into<String>) -> Self {
        self.body = body.into();
        self
    }

    pub fn with_remote_addr(mut self, addr: impl Into<String>) -> Self {
        self.remote_addr = addr.into();
        self
    }

    pub fn with_host(mut self, host: impl Into<String>) -> Self {
        self.host = host.into();
        self
    }
}

impl SaRequest for SaRequestForMock {
    fn source(&self) -> &dyn std::any::Any {
        self
    }

    fn get_param(&self, name: &str) -> Option<String> {
        self.query.get(name).cloned()
    }

    fn get_header(&self, name: &str) -> Option<String> {
        // 大小写不敏感
        let lk = name.to_ascii_lowercase();
        self.headers
            .iter()
            .find(|(k, _)| k.to_ascii_lowercase() == lk)
            .map(|(_, v)| v.clone())
    }

    fn get_cookie_value(&self, name: &str) -> Option<String> {
        // 在 mock 中，cookie 一般通过 header 注入
        self.headers.get("cookie").and_then(|raw| {
            raw.split(';').map(str::trim).find_map(|kv| {
                let mut it = kv.splitn(2, '=');
                let k = it.next()?.trim();
                let v = it.next()?.trim();
                if k == name { Some(v.to_string()) } else { None }
            })
        })
    }

    fn get_request_path(&self) -> String {
        // 简单实现：去掉 query string
        self.url.split('?').next().unwrap_or(&self.url).to_string()
    }

    fn get_url(&self) -> String {
        self.url.clone()
    }

    fn get_method(&self) -> String {
        if self.method.is_empty() {
            "GET".into()
        } else {
            self.method.clone()
        }
    }

    fn get_host(&self) -> String {
        if self.host.is_empty() {
            // 简单解析：取 URL 中的 host 部分
            if let Some(scheme_end) = self.url.find("://") {
                let after = &self.url[scheme_end + 3..];
                after.split('/').next().unwrap_or("").to_string()
            } else {
                String::new()
            }
        } else {
            self.host.clone()
        }
    }

    fn is_ajax(&self) -> bool {
        self.get_header("x-requested-with")
            .map(|v| v.eq_ignore_ascii_case("XMLHttpRequest"))
            .unwrap_or(false)
    }

    fn forward(&self, path: &str) {
        // Mock：打印即可
        eprintln!("[SaRequestForMock] forward -> {path}");
    }
}

/// 实现一个 get_cookies 辅助方法（mock 解析 cookies）
pub fn cookies_for(mock: &SaRequestForMock) -> Vec<SaCookie> {
    let mut out = Vec::new();
    let raw = match mock.headers.get("cookie") {
        Some(s) => s,
        None => return out,
    };
    for kv in raw.split(';').map(str::trim) {
        if kv.is_empty() {
            continue;
        }
        let (k, v) = match kv.split_once('=') {
            Some(kv) => kv,
            None => continue,
        };
        out.push(SaCookie {
            name: k.to_string(),
            value: v.to_string(),
            max_age: -1,
            domain: String::new(),
            path: "/".to_string(),
            secure: false,
            http_only: false,
            same_site: "Lax".to_string(),
            extra_attrs: HashMap::new(),
        });
    }
    out
}
