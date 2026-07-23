//! Cookie 模型（对应 Java `cn.dev33.satoken.context.model.SaCookie`）。

use std::collections::HashMap;

use crate::error::SaErrorCode;
use crate::exception::SaTokenException;

/// 写入响应头时使用的 key（对应 Java `HEADER_NAME`）
pub const HEADER_NAME: &str = "Set-Cookie";

/// Cookie 模型
#[derive(Debug, Clone, Default)]
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
    /// 额外扩展属性
    pub extra_attrs: HashMap<String, String>,
}

impl SaCookie {
    /// 创建 Cookie
    pub fn new(name: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            value: value.into(),
            max_age: -1,
            domain: String::new(),
            path: String::new(),
            secure: false,
            http_only: false,
            same_site: String::new(),
            extra_attrs: HashMap::new(),
        }
    }

    /// 设置名称
    pub fn set_name(&mut self, name: impl Into<String>) -> &mut Self {
        self.name = name.into();
        self
    }

    /// 设置值
    pub fn set_value(&mut self, value: impl Into<String>) -> &mut Self {
        self.value = value.into();
        self
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

    /// 设置 SameSite
    pub fn set_same_site(&mut self, same_site: impl Into<String>) -> &mut Self {
        self.same_site = same_site.into();
        self
    }

    /// 追加扩展属性
    pub fn add_extra_attr(
        &mut self,
        name: impl Into<String>,
        value: impl Into<String>,
    ) -> &mut Self {
        self.extra_attrs.insert(name.into(), value.into());
        self
    }

    /// 填充默认值（对应 Java `builder()`）
    pub fn builder(&mut self) {
        if self.path.is_empty() {
            self.path = "/".to_string();
        }
    }

    /// 转换为 HTTP 响应头格式（对应 Java `toHeaderValue()`）
    ///
    /// # Errors
    /// name 为空或 value 含 `;` 时返回 [`SaTokenException`]
    pub fn to_header_value(&self) -> Result<String, SaTokenException> {
        let mut cookie = self.clone();
        cookie.builder();

        if cookie.name.is_empty() {
            return Err(SaTokenException::with_code(
                SaErrorCode::CODE_12002,
                "name不能为空",
            ));
        }
        if cookie.value.contains(';') {
            return Err(SaTokenException::with_code(
                SaErrorCode::CODE_12003,
                format!("无效Value：{}", cookie.value),
            ));
        }

        let mut parts = vec![format!("{}={}", cookie.name, cookie.value)];
        if cookie.max_age >= 0 {
            parts.push(format!("Max-Age={}", cookie.max_age));
        }
        if !cookie.domain.is_empty() {
            parts.push(format!("Domain={}", cookie.domain));
        }
        if !cookie.path.is_empty() {
            parts.push(format!("Path={}", cookie.path));
        }
        if cookie.secure {
            parts.push("Secure".to_string());
        }
        if cookie.http_only {
            parts.push("HttpOnly".to_string());
        }
        if !cookie.same_site.is_empty() {
            parts.push(format!("SameSite={}", cookie.same_site));
        }
        for (key, value) in &cookie.extra_attrs {
            if value.is_empty() {
                parts.push(key.clone());
            } else {
                parts.push(format!("{key}={value}"));
            }
        }
        Ok(parts.join("; "))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn header_value_contains_core_fields() {
        let mut cookie = SaCookie::new("satoken", "abc");
        cookie
            .set_path("/api")
            .set_same_site("Lax")
            .set_http_only(true);
        let header = cookie.to_header_value().expect("valid cookie");
        assert!(header.contains("satoken=abc"));
        assert!(header.contains("Path=/api"));
        assert!(header.contains("SameSite=Lax"));
        assert!(header.contains("HttpOnly"));
    }

    #[test]
    fn empty_name_is_rejected() {
        let cookie = SaCookie::default();
        let err = cookie.to_header_value().expect_err("must fail");
        assert_eq!(err.code(), SaErrorCode::CODE_12002);
    }
}
