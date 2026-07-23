//! `SaHttpBasicAccount` —— 1:1 对应 Java `cn.dev33.satoken.httpauth.basic.SaHttpBasicAccount`
//!
//! HTTP Basic 认证账号模型。

use crate::exception::SaTokenException;
use crate::util::sa_fox_util::SaFoxUtil;
use serde::{Deserialize, Serialize};

/// HTTP Basic 账号模型（对应 Java `SaHttpBasicAccount`）。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SaHttpBasicAccount {
    /// 用户名
    pub username: String,
    /// 密码
    pub password: String,
}

impl SaHttpBasicAccount {
    /// 通过用户名和密码创建（对应 Java 构造 `SaHttpBasicAccount(username, password)`）。
    pub fn new(username: impl Into<String>, password: impl Into<String>) -> Self {
        Self {
            username: username.into(),
            password: password.into(),
        }
    }

    /// 通过 `username:password` 字符串创建（对应 Java 构造 `SaHttpBasicAccount(String)`）。
    pub fn from_username_and_password(value: &str) -> Result<Self, SaTokenException> {
        if SaFoxUtil::is_empty(value) {
            return Err(SaTokenException::with_code(
                crate::error::SaErrorCode::CODE_10001,
                "UsernameAndPassword 不能为空",
            ));
        }
        let Some((username, password)) = value.split_once(':') else {
            return Err(SaTokenException::with_code(
                crate::error::SaErrorCode::CODE_10001,
                "UsernameAndPassword 格式错误，正确格式为：username:password",
            ));
        };
        Ok(Self {
            username: username.to_string(),
            password: password.to_string(),
        })
    }

    /// 格式化为 `username:password`（用于与 Authorization 解码值比对）。
    pub fn to_credential_string(&self) -> String {
        format!("{}:{}", self.username, self.password)
    }
}
