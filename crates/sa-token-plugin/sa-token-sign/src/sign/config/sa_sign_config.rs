use std::sync::Arc;

use md5::Md5;
use sha1::Sha1;
use sha2::{Digest, Sha256, Sha384, Sha512};

use crate::sign::exception::SaSignException;

type DigestMethod = Arc<dyn Fn(&str) -> Result<String, SaSignException> + Send + Sync>;

/// Signature configuration and digest policy.
#[derive(Clone)]
pub struct SaSignConfig {
    pub secret_key: String,
    pub timestamp_disparity: i64,
    pub digest_algo: String,
    digest_method: Option<DigestMethod>,
}

impl std::fmt::Debug for SaSignConfig {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("SaSignConfig")
            .field("secret_key", &"[REDACTED]")
            .field("timestamp_disparity", &self.timestamp_disparity)
            .field("digest_algo", &self.digest_algo)
            .finish()
    }
}

impl Default for SaSignConfig {
    fn default() -> Self {
        Self {
            secret_key: String::new(),
            timestamp_disparity: 900_000,
            digest_algo: "md5".into(),
            digest_method: None,
        }
    }
}

impl SaSignConfig {
    pub fn new(secret_key: impl Into<String>) -> Self {
        Self {
            secret_key: secret_key.into(),
            ..Self::default()
        }
    }
    pub fn save_nonce_expire(&self) -> i64 {
        if self.timestamp_disparity >= 0 {
            self.timestamp_disparity / 1000
        } else {
            86_400
        }
    }
    pub fn with_timestamp_disparity(mut self, value: i64) -> Self {
        self.timestamp_disparity = value;
        self
    }
    pub fn with_digest_algo(mut self, value: impl Into<String>) -> Self {
        self.digest_algo = value.into();
        self
    }
    pub fn with_digest_method(mut self, value: DigestMethod) -> Self {
        self.digest_method = Some(value);
        self
    }
    pub fn digest(&self, value: &str) -> Result<String, SaSignException> {
        if let Some(method) = &self.digest_method {
            return method(value);
        }
        let bytes = value.as_bytes();
        let encoded = match self.digest_algo.to_ascii_lowercase().as_str() {
            "md5" => hex::encode(Md5::digest(bytes)),
            "sha1" => hex::encode(Sha1::digest(bytes)),
            "sha256" => hex::encode(Sha256::digest(bytes)),
            "sha384" => hex::encode(Sha384::digest(bytes)),
            "sha512" => hex::encode(Sha512::digest(bytes)),
            algorithm => {
                return Err(SaSignException::new(
                    0,
                    format!("unsupported digest algorithm: {algorithm}"),
                ));
            }
        };
        Ok(encoded)
    }
}
