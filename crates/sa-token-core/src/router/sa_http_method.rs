//! HTTP 方法枚举（对应 Java `cn.dev33.satoken.router.SaHttpMethod`）。

use crate::error::SaErrorCode;
use crate::exception::{SaResult, SaTokenException};

/// HTTP 请求方法。
///
/// 枚举成员遵循 Rust 的 `PascalCase` 命名；其文本表示与 Java 枚举保持一致。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SaHttpMethod {
    /// GET
    Get,
    /// HEAD
    Head,
    /// POST
    Post,
    /// PUT
    Put,
    /// PATCH
    Patch,
    /// DELETE
    Delete,
    /// OPTIONS
    Options,
    /// TRACE
    Trace,
    /// CONNECT
    Connect,
    /// 匹配全部请求方法。
    All,
}

impl SaHttpMethod {
    /// 从字符串解析请求方法。
    ///
    /// # Errors
    ///
    /// 方法为空或不是受支持的 HTTP 方法时返回 Java 兼容错误码 `10321`。
    pub fn parse(method: &str) -> SaResult<Self> {
        if method.trim().is_empty() {
            return Err(Self::invalid_method(method));
        }

        match method.to_uppercase().as_str() {
            "GET" => Ok(Self::Get),
            "HEAD" => Ok(Self::Head),
            "POST" => Ok(Self::Post),
            "PUT" => Ok(Self::Put),
            "PATCH" => Ok(Self::Patch),
            "DELETE" => Ok(Self::Delete),
            "OPTIONS" => Ok(Self::Options),
            "TRACE" => Ok(Self::Trace),
            "CONNECT" => Ok(Self::Connect),
            "ALL" => Ok(Self::All),
            _ => Err(Self::invalid_method(method)),
        }
    }

    /// 批量解析请求方法。
    ///
    /// # Errors
    ///
    /// 任一方法无效时立即返回错误码 `10321`。
    pub fn parse_all(methods: &[&str]) -> SaResult<Vec<Self>> {
        methods.iter().map(|method| Self::parse(method)).collect()
    }

    /// 是否匹配另一个请求方法。
    pub fn matches(&self, other: &SaHttpMethod) -> bool {
        *self == Self::All || *other == Self::All || *self == *other
    }

    /// 是否是 `ALL`。
    pub fn is_all(&self) -> bool {
        matches!(self, Self::All)
    }

    /// 是否有任意一个请求方法与当前值匹配。
    pub fn is_any(&self, others: &[SaHttpMethod]) -> bool {
        others.iter().any(|m| self.matches(m))
    }

    fn invalid_method(method: &str) -> SaTokenException {
        SaTokenException::with_code(SaErrorCode::CODE_10321, format!("无效 Method：{method}"))
    }
}

impl std::fmt::Display for SaHttpMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Get => write!(f, "GET"),
            Self::Head => write!(f, "HEAD"),
            Self::Post => write!(f, "POST"),
            Self::Put => write!(f, "PUT"),
            Self::Patch => write!(f, "PATCH"),
            Self::Delete => write!(f, "DELETE"),
            Self::Options => write!(f, "OPTIONS"),
            Self::Trace => write!(f, "TRACE"),
            Self::Connect => write!(f, "CONNECT"),
            Self::All => write!(f, "ALL"),
        }
    }
}

impl std::str::FromStr for SaHttpMethod {
    type Err = SaTokenException;

    fn from_str(method: &str) -> Result<Self, Self::Err> {
        Self::parse(method)
    }
}

#[cfg(test)]
mod tests {
    use super::SaHttpMethod;
    use crate::error::SaErrorCode;

    #[test]
    fn parses_every_java_http_method_case_insensitively() {
        let cases = [
            ("get", SaHttpMethod::Get),
            ("HEAD", SaHttpMethod::Head),
            ("post", SaHttpMethod::Post),
            ("PUT", SaHttpMethod::Put),
            ("patch", SaHttpMethod::Patch),
            ("DELETE", SaHttpMethod::Delete),
            ("options", SaHttpMethod::Options),
            ("TRACE", SaHttpMethod::Trace),
            ("connect", SaHttpMethod::Connect),
            ("ALL", SaHttpMethod::All),
        ];

        for (raw, expected) in cases {
            assert_eq!(SaHttpMethod::parse(raw).expect("valid method"), expected);
        }
    }

    #[test]
    fn rejects_unknown_method_instead_of_turning_it_into_all() {
        let error = SaHttpMethod::parse("BREW").expect_err("unknown method must fail");
        assert_eq!(error.code(), SaErrorCode::CODE_10321);
        assert!(SaHttpMethod::parse("").is_err());
    }

    #[test]
    fn all_matches_every_method() {
        assert!(SaHttpMethod::All.matches(&SaHttpMethod::Connect));
        assert!(SaHttpMethod::Get.is_any(&[SaHttpMethod::Post, SaHttpMethod::All]));
    }
}
