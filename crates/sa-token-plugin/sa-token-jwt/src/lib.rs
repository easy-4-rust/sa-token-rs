//! Sa-Token JWT 插件
//!
//! 对应 Java Sa-Token 的 `sa-token-jwt` 插件，提供 JWT Token 生成与验证。
//!
//! # 示例
//!
//! ```rust,ignore
//! use sa_token_jwt::{SaJwtTemplate, JwtConfig};
//!
//! let config = JwtConfig::new("my-secret-key");
//! let jwt = SaJwtTemplate::new(config);
//!
//! // 生成 Token
//! let token = jwt.create_token("10001", "login", 3600).unwrap();
//!
//! // 解析 Token
//! let claims = jwt.parse_token(&token).unwrap();
//! assert_eq!(claims.sub, "10001");
//! ```

use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// JWT 配置
#[derive(Debug, Clone)]
pub struct JwtConfig {
    /// 密钥
    pub secret_key: String,
    /// 算法
    pub algorithm: Algorithm,
    /// 签发者
    pub issuer: Option<String>,
    /// 受众
    pub audience: Option<String>,
}

impl JwtConfig {
    /// 创建新的 JWT 配置
    pub fn new(secret_key: impl Into<String>) -> Self {
        Self {
            secret_key: secret_key.into(),
            algorithm: Algorithm::HS256,
            issuer: None,
            audience: None,
        }
    }

    /// 设置算法
    pub fn with_algorithm(mut self, algorithm: Algorithm) -> Self {
        self.algorithm = algorithm;
        self
    }

    /// 设置签发者
    pub fn with_issuer(mut self, issuer: impl Into<String>) -> Self {
        self.issuer = Some(issuer.into());
        self
    }

    /// 设置受众
    pub fn with_audience(mut self, audience: impl Into<String>) -> Self {
        self.audience = Some(audience.into());
        self
    }
}

/// JWT Claims
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaJwtClaims {
    /// 主题（login_id）
    pub sub: String,
    /// 账号类型
    pub login_type: String,
    /// 设备类型
    pub device: String,
    /// 签发时间
    pub iat: u64,
    /// 过期时间
    pub exp: u64,
    /// 签发者
    #[serde(skip_serializing_if = "Option::is_none")]
    pub iss: Option<String>,
    /// 受众
    #[serde(skip_serializing_if = "Option::is_none")]
    pub aud: Option<String>,
    /// 扩展数据
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extra: Option<HashMap<String, serde_json::Value>>,
}

/// JWT Token 信息
#[derive(Debug, Clone)]
pub struct SaJwtTokenInfo {
    /// Token 值
    pub token: String,
    /// Claims
    pub claims: SaJwtClaims,
}

/// JWT 模板
pub struct SaJwtTemplate {
    /// 配置
    config: JwtConfig,
}

impl SaJwtTemplate {
    /// 创建新的 JWT 模板
    pub fn new(config: JwtConfig) -> Self {
        Self { config }
    }

    /// 创建 JWT Token
    pub fn create_token(
        &self,
        login_id: &str,
        login_type: &str,
        timeout: i64,
    ) -> Result<String, SaJwtError> {
        self.create_token_with_extra(login_id, login_type, timeout, None)
    }

    /// 创建带扩展数据的 JWT Token
    pub fn create_token_with_extra(
        &self,
        login_id: &str,
        login_type: &str,
        timeout: i64,
        extra: Option<HashMap<String, serde_json::Value>>,
    ) -> Result<String, SaJwtError> {
        let now = chrono::Utc::now().timestamp() as u64;
        let exp = if timeout > 0 {
            now + timeout as u64
        } else {
            u64::MAX
        };

        let claims = SaJwtClaims {
            sub: login_id.to_string(),
            login_type: login_type.to_string(),
            device: String::new(),
            iat: now,
            exp,
            iss: self.config.issuer.clone(),
            aud: self.config.audience.clone(),
            extra,
        };

        let header = Header {
            alg: self.config.algorithm,
            ..Default::default()
        };

        let encoding_key = EncodingKey::from_secret(self.config.secret_key.as_bytes());
        encode(&header, &claims, &encoding_key).map_err(|e| SaJwtError::EncodeError(e.to_string()))
    }

    /// 解析 JWT Token
    pub fn parse_token(&self, token: &str) -> Result<SaJwtClaims, SaJwtError> {
        let decoding_key = DecodingKey::from_secret(self.config.secret_key.as_bytes());
        let mut validation = Validation::new(self.config.algorithm);

        if let Some(ref issuer) = self.config.issuer {
            validation.set_issuer(&[issuer]);
        }
        if let Some(ref audience) = self.config.audience {
            validation.set_audience(&[audience]);
        }

        let token_data =
            decode::<SaJwtClaims>(token, &decoding_key, &validation).map_err(|e| {
                match e.kind() {
                    jsonwebtoken::errors::ErrorKind::ExpiredSignature => {
                        SaJwtError::TokenExpired
                    }
                    _ => SaJwtError::DecodeError(e.to_string()),
                }
            })?;

        Ok(token_data.claims)
    }

    /// 验证 Token 是否有效
    pub fn verify_token(&self, token: &str) -> bool {
        self.parse_token(token).is_ok()
    }

    /// 获取 Token 中的 login_id
    pub fn get_login_id(&self, token: &str) -> Option<String> {
        self.parse_token(token).ok().map(|c| c.sub)
    }

    /// 获取 Token 中的过期时间
    pub fn get_exp(&self, token: &str) -> Option<u64> {
        self.parse_token(token).ok().map(|c| c.exp)
    }

    /// 判断 Token 是否已过期
    pub fn is_expired(&self, token: &str) -> bool {
        match self.parse_token(token) {
            Ok(claims) => {
                let now = chrono::Utc::now().timestamp() as u64;
                claims.exp <= now
            }
            Err(SaJwtError::TokenExpired) => true,
            Err(_) => false,
        }
    }
}

/// JWT 错误类型
#[derive(Debug, thiserror::Error)]
pub enum SaJwtError {
    /// Token 编码错误
    #[error("Token 编码错误: {0}")]
    EncodeError(String),

    /// Token 解码错误
    #[error("Token 解码错误: {0}")]
    DecodeError(String),

    /// Token 已过期
    #[error("Token 已过期")]
    TokenExpired,
}
