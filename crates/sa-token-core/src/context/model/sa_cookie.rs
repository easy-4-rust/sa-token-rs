//! Cookie 模型（对应 Java `cn.dev33.satoken.context.model.SaCookie`）。

/// Cookie 模型
#[derive(Debug, Clone)]
pub struct SaCookie {
    /// Cookie 名称
    pub name: String,
    /// Cookie 值
    pub value: String,
    /// Cookie 超时时间（秒），-1 代表会话级
    pub max_age: i64,
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

impl SaCookie {
    /// 创建 Cookie
    pub fn new(name: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            value: value.into(),
            max_age: -1,
            domain: String::new(),
            path: "/".to_string(),
            secure: false,
            http_only: true,
            same_site: "Lax".to_string(),
        }
    }

    /// 设置超时时间
    pub fn set_max_age(&mut self, max_age: i64) -> &mut Self {
        self.max_age = max_age;
        self
    }

    /// 设置域名
    pub fn set_domain(&mut self, domain: impl Into<String>) -> &mut Self {
        self.domain = domain.into();
        self
    }

    /// 设置路径
    pub fn set_path(&mut self, path: impl Into<String>) -> &mut Self {
        self.path = path.into();
        self
    }

    /// 设置是否仅 HTTPS
    pub fn set_secure(&mut self, secure: bool) -> &mut Self {
        self.secure = secure;
        self
    }

    /// 设置是否仅 HTTP 访问
    pub fn set_http_only(&mut self, http_only: bool) -> &mut Self {
        self.http_only = http_only;
        self
    }

    /// 转换为 HTTP 响应头格式
    pub fn to_header_value(&self) -> String {
        let mut parts = vec![format!("{}={}", self.name, self.value)];
        if !self.domain.is_empty() {
            parts.push(format!("Domain={}", self.domain));
        }
        parts.push(format!("Path={}", self.path));
        if self.max_age >= 0 {
            parts.push(format!("Max-Age={}", self.max_age));
        }
        if self.secure {
            parts.push("Secure".to_string());
        }
        if self.http_only {
            parts.push("HttpOnly".to_string());
        }
        if !self.same_site.is_empty() {
            parts.push(format!("SameSite={}", self.same_site));
        }
        parts.join("; ")
    }
}
