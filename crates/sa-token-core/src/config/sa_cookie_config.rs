//! Cookie 配置（对应 Java `cn.dev33.satoken.config.SaCookieConfig`）。
use serde::{Deserialize, Serialize};

/// Cookie 配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaCookieConfig {
    /// Cookie 域名
    pub domain: String,
    /// Cookie 路径
    pub path: String,
    /// 是否仅 HTTPS
    pub secure: bool,
    /// 是否仅 HTTP 访问
    pub http_only: bool,
    /// SameSite 属性
    pub same_site: String,
}

impl Default for SaCookieConfig {
    fn default() -> Self {
        Self {
            domain: String::new(),
            path: "/".to_string(),
            secure: false,
            http_only: true,
            same_site: "Lax".to_string(),
        }
    }
}

impl SaCookieConfig {
    /// 获取域名
    pub fn domain(&self) -> &str {
        &self.domain
    }

    /// 设置域名
    pub fn set_domain(&mut self, domain: impl Into<String>) {
        self.domain = domain.into();
    }

    /// 获取路径
    pub fn path(&self) -> &str {
        &self.path
    }

    /// 设置路径
    pub fn set_path(&mut self, path: impl Into<String>) {
        self.path = path.into();
    }

    /// 是否仅 HTTPS
    pub fn is_secure(&self) -> bool {
        self.secure
    }

    /// 设置是否仅 HTTPS
    pub fn set_secure(&mut self, secure: bool) {
        self.secure = secure;
    }

    /// 是否仅 HTTP 访问
    pub fn is_http_only(&self) -> bool {
        self.http_only
    }

    /// 设置是否仅 HTTP 访问
    pub fn set_http_only(&mut self, http_only: bool) {
        self.http_only = http_only;
    }

    /// 获取 SameSite 属性
    pub fn same_site(&self) -> &str {
        &self.same_site
    }

    /// 设置 SameSite 属性
    pub fn set_same_site(&mut self, same_site: impl Into<String>) {
        self.same_site = same_site.into();
    }
}
