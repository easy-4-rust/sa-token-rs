//! `SaResponseForMock` —— 1:1 对应 Java `cn.dev33.satoken.context.model.SaResponseForMock`

use super::super::model::sa_cookie::SaCookie;
use super::super::model::sa_response::SaResponse;
use std::collections::HashMap;
use std::sync::Mutex;

/// `SaResponse` 的 Mock 实现
#[derive(Debug)]
pub struct SaResponseForMock {
    inner: Mutex<MockInner>,
}

#[derive(Debug, Default)]
struct MockInner {
    headers: HashMap<String, String>,
    cookies: Vec<(String, String)>,
    body: String,
    status: u16,
}

impl SaResponseForMock {
    pub fn new() -> Self {
        Self {
            inner: Mutex::new(MockInner {
                status: 200,
                ..Default::default()
            }),
        }
    }

    pub fn with_status(self, status: u16) -> Self {
        self.inner.lock().unwrap().status = status;
        self
    }

    pub fn with_header(self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.inner
            .lock()
            .unwrap()
            .headers
            .insert(key.into(), value.into());
        self
    }

    pub fn with_body(self, body: impl Into<String>) -> Self {
        self.inner.lock().unwrap().body = body.into();
        self
    }
}

impl Default for SaResponseForMock {
    fn default() -> Self {
        Self::new()
    }
}

impl SaResponse for SaResponseForMock {
    fn source(&self) -> &dyn std::any::Any {
        self
    }

    fn set_status(&self, sc: u16) {
        self.inner.lock().unwrap().status = sc;
    }

    fn set_header(&self, name: &str, value: &str) {
        self.inner
            .lock()
            .unwrap()
            .headers
            .insert(name.to_string(), value.to_string());
    }
    fn add_header(&self, name: &str, value: &str) {
        // Mock：相同 key 追加为单值
        self.inner
            .lock()
            .unwrap()
            .headers
            .insert(name.to_string(), value.to_string());
    }
    fn add_cookie(&self, cookie: SaCookie) {
        self.inner
            .lock()
            .unwrap()
            .cookies
            .push((cookie.name, cookie.value));
    }
    fn delete_cookie(&self, name: &str) {
        self.inner
            .lock()
            .unwrap()
            .cookies
            .retain(|(k, _)| k != name);
    }
    fn redirect(&self, url: &str) {
        // Mock：打印即可
        eprintln!("[SaResponseForMock] redirect -> {url}");
    }
}

/// Mock 上的便捷读取方法
impl SaResponseForMock {
    pub fn body(&self) -> String {
        self.inner.lock().unwrap().body.clone()
    }
    pub fn status(&self) -> u16 {
        self.inner.lock().unwrap().status
    }
    pub fn cookie(&self, name: &str) -> Option<String> {
        self.inner
            .lock()
            .unwrap()
            .cookies
            .iter()
            .find(|(k, _)| k == name)
            .map(|(_, v)| v.clone())
    }
    pub fn header(&self, name: &str) -> Option<String> {
        self.inner.lock().unwrap().headers.get(name).cloned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_flow() {
        let r = SaResponseForMock::new()
            .with_status(201)
            .with_header("X-Foo", "bar");
        r.set_status(202);
        assert_eq!(r.status(), 202);
        assert_eq!(r.header("X-Foo"), Some("bar".to_string()));
        r.set_header("X-Foo", "baz");
        assert_eq!(r.header("X-Foo"), Some("baz".to_string()));
    }
}
