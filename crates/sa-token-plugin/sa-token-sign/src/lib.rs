//! Sa-Token Sign 插件
//!
//! 对应 Java Sa-Token 的 `sa-token-sign` 插件，提供 API 参数签名校验。
//!
//! # 示例
//!
//! ```rust,ignore
//! use sa_token_sign::{SaSignTemplate, SignConfig};
//!
//! let config = SignConfig::new("my-secret-key");
//! let sign = SaSignTemplate::new(config);
//!
//! // 生成签名
//! let params = vec![("name", "value"), ("age", "25")];
//! let signature = sign.create_sign(&params);
//!
//! // 验证签名
//! assert!(sign.verify_sign(&params, &signature));
//! ```

use sha2::{Digest, Sha256};

/// 签名配置
#[derive(Debug, Clone)]
pub struct SignConfig {
    /// 密钥
    pub secret_key: String,
    /// 排序方式（true=字典序，false=原始顺序）
    pub sort: bool,
    /// 分隔符
    pub separator: String,
}

impl SignConfig {
    /// 创建新的签名配置
    pub fn new(secret_key: impl Into<String>) -> Self {
        Self {
            secret_key: secret_key.into(),
            sort: true,
            separator: "&".to_string(),
        }
    }

    /// 设置排序方式
    pub fn with_sort(mut self, sort: bool) -> Self {
        self.sort = sort;
        self
    }

    /// 设置分隔符
    pub fn with_separator(mut self, separator: impl Into<String>) -> Self {
        self.separator = separator.into();
        self
    }
}

/// 签名模板
pub struct SaSignTemplate {
    /// 配置
    config: SignConfig,
}

impl SaSignTemplate {
    /// 创建新的签名模板
    pub fn new(config: SignConfig) -> Self {
        Self { config }
    }

    /// 创建签名
    pub fn create_sign(&self, params: &[(&str, &str)]) -> String {
        let sign_str = self.build_sign_string(params);
        self.sign(&sign_str)
    }

    /// 创建签名（带额外参数）
    pub fn create_sign_with_timestamp(
        &self,
        params: &[(&str, &str)],
        timestamp: i64,
    ) -> String {
        let mut params_with_ts: Vec<(&str, &str)> = params.to_vec();
        let ts_str = timestamp.to_string();
        params_with_ts.push(("timestamp", &ts_str));
        self.create_sign(&params_with_ts)
    }

    /// 验证签名
    pub fn verify_sign(&self, params: &[(&str, &str)], signature: &str) -> bool {
        let expected = self.create_sign(params);
        expected == signature
    }

    /// 验证签名（带时间戳校验）
    pub fn verify_sign_with_timestamp(
        &self,
        params: &[(&str, &str)],
        signature: &str,
        timestamp: i64,
        max_age: i64,
    ) -> Result<bool, SaSignError> {
        // 检查时间戳是否在有效期内
        let now = chrono::Utc::now().timestamp();
        if (now - timestamp).abs() > max_age {
            return Err(SaSignError::TimestampExpired);
        }

        Ok(self.verify_sign(params, signature))
    }

    /// 构建签名字符串
    fn build_sign_string(&self, params: &[(&str, &str)]) -> String {
        let mut sorted_params: Vec<(&str, &str)> = params.to_vec();

        if self.config.sort {
            sorted_params.sort_by(|a, b| a.0.cmp(b.0));
        }

        let param_str: Vec<String> = sorted_params
            .iter()
            .filter(|(k, _)| !k.is_empty())
            .map(|(k, v)| format!("{}={}", k, v))
            .collect();

        format!(
            "{}{}{}",
            param_str.join(&self.config.separator),
            self.config.separator,
            self.config.secret_key
        )
    }

    /// 执行签名
    fn sign(&self, content: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(content.as_bytes());
        let result = hasher.finalize();
        hex::encode(result)
    }
}

/// 签名错误类型
#[derive(Debug, thiserror::Error)]
pub enum SaSignError {
    /// 时间戳已过期
    #[error("时间戳已过期")]
    TimestampExpired,
}
