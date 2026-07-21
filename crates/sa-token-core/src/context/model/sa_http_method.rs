//! HTTP 方法枚举（对应 Java `cn.dev33.satoken.router.SaHttpMethod`）。

/// HTTP 请求方法
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SaHttpMethod {
    /// GET
    GET,
    /// POST
    POST,
    /// PUT
    PUT,
    /// DELETE
    DELETE,
    /// PATCH
    PATCH,
    /// HEAD
    HEAD,
    /// OPTIONS
    OPTIONS,
    /// TRACE
    TRACE,
    /// ANY（匹配所有方法）
    ANY,
}

impl SaHttpMethod {
    /// 从字符串解析
    pub fn from_str(method: &str) -> Self {
        match method.to_uppercase().as_str() {
            "GET" => Self::GET,
            "POST" => Self::POST,
            "PUT" => Self::PUT,
            "DELETE" => Self::DELETE,
            "PATCH" => Self::PATCH,
            "HEAD" => Self::HEAD,
            "OPTIONS" => Self::OPTIONS,
            "TRACE" => Self::TRACE,
            _ => Self::ANY,
        }
    }

    /// 是否匹配
    pub fn matches(&self, other: &SaHttpMethod) -> bool {
        *self == Self::ANY || *other == Self::ANY || *self == *other
    }
}

impl std::fmt::Display for SaHttpMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::GET => write!(f, "GET"),
            Self::POST => write!(f, "POST"),
            Self::PUT => write!(f, "PUT"),
            Self::DELETE => write!(f, "DELETE"),
            Self::PATCH => write!(f, "PATCH"),
            Self::HEAD => write!(f, "HEAD"),
            Self::OPTIONS => write!(f, "OPTIONS"),
            Self::TRACE => write!(f, "TRACE"),
            Self::ANY => write!(f, "ANY"),
        }
    }
}
