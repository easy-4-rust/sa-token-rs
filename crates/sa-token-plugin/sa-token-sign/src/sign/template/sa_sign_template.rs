use std::collections::{BTreeMap, HashMap};
use std::sync::Arc;

use sa_token_core::dao::sa_token_dao::SaTokenDao;
use sa_token_core::util::sa_fox_util::{now_timestamp_millis, random_string};

use crate::sign::{SaSignConfig, SaSignErrorCode, SaSignException};

/// Request-signature algorithm and replay protection boundary.
pub struct SaSignTemplate {
    pub config: Arc<SaSignConfig>,
    dao: Arc<dyn SaTokenDao>,
    token_name: String,
}

impl SaSignTemplate {
    pub const KEY: &'static str = "key";
    pub const TIMESTAMP: &'static str = "timestamp";
    pub const NONCE: &'static str = "nonce";
    pub const SIGN: &'static str = "sign";

    pub fn new(
        config: Arc<SaSignConfig>,
        dao: Arc<dyn SaTokenDao>,
        token_name: impl Into<String>,
    ) -> Self {
        Self {
            config,
            dao,
            token_name: token_name.into(),
        }
    }

    pub fn join_params(params: &BTreeMap<String, String>) -> String {
        params
            .iter()
            .filter(|(_, value)| !value.is_empty())
            .map(|(key, value)| format!("{key}={value}"))
            .collect::<Vec<_>>()
            .join("&")
    }

    pub fn create_sign(&self, params: &HashMap<String, String>) -> Result<String, SaSignException> {
        if self.config.secret_key.is_empty() {
            return Err(SaSignException::new(
                SaSignErrorCode::CODE_12201,
                "signature secret must not be empty",
            ));
        }
        let sorted: BTreeMap<String, String> = params
            .iter()
            .filter(|(key, _)| key.as_str() != Self::SIGN)
            .map(|(key, value)| (key.clone(), value.clone()))
            .collect();
        let full = format!(
            "{}&{}={}",
            Self::join_params(&sorted),
            Self::KEY,
            self.config.secret_key
        );
        self.config.digest(&full)
    }

    pub fn add_sign_params(
        &self,
        params: &mut HashMap<String, String>,
    ) -> Result<(), SaSignException> {
        params.insert(Self::TIMESTAMP.into(), now_timestamp_millis().to_string());
        params.insert(Self::NONCE.into(), random_string(32));
        params.insert(Self::SIGN.into(), self.create_sign(params)?);
        Ok(())
    }

    pub fn is_valid_timestamp(&self, timestamp: i64) -> bool {
        self.config.timestamp_disparity == -1
            || now_timestamp_millis().abs_diff(timestamp) <= self.config.timestamp_disparity as u64
    }

    pub fn check_timestamp(&self, timestamp: i64) -> Result<(), SaSignException> {
        self.is_valid_timestamp(timestamp)
            .then_some(())
            .ok_or_else(|| {
                SaSignException::new(
                    SaSignErrorCode::CODE_12203,
                    "timestamp exceeds allowed disparity",
                )
            })
    }

    pub fn nonce_key(&self, nonce: &str) -> String {
        format!("{}:sign:nonce:{nonce}", self.token_name)
    }

    pub fn is_valid_nonce(&self, nonce: &str) -> Result<bool, SaSignException> {
        if nonce.is_empty() {
            return Ok(false);
        }
        self.dao
            .get(&self.nonce_key(nonce))
            .map(|value| value.is_none())
            .map_err(|error| SaSignException::new(0, error.to_string()))
    }

    pub fn check_nonce(&self, nonce: &str) -> Result<(), SaSignException> {
        if !self.is_valid_nonce(nonce)? {
            return Err(SaSignException::new(
                0,
                "nonce is empty or has already been used",
            ));
        }
        self.dao
            .set(
                &self.nonce_key(nonce),
                nonce,
                self.config.save_nonce_expire() * 2 + 2,
            )
            .map_err(|error| SaSignException::new(0, error.to_string()))
    }

    pub fn check_sign(
        &self,
        params: &HashMap<String, String>,
        signature: &str,
    ) -> Result<(), SaSignException> {
        // Constant-time comparison is provided by the equality helper in the HMAC ecosystem only;
        // digest signatures here retain Java compatibility and never log either secret or full string.
        if self.create_sign(params)? == signature {
            Ok(())
        } else {
            Err(SaSignException::new(
                SaSignErrorCode::CODE_12202,
                "invalid signature",
            ))
        }
    }

    pub fn check_param_map(&self, params: &HashMap<String, String>) -> Result<(), SaSignException> {
        let timestamp = params
            .get(Self::TIMESTAMP)
            .filter(|value| !value.is_empty())
            .ok_or_else(|| SaSignException::new(0, "missing timestamp"))?
            .parse::<i64>()
            .map_err(|error| SaSignException::new(0, error.to_string()))?;
        let nonce = params
            .get(Self::NONCE)
            .filter(|value| !value.is_empty())
            .ok_or_else(|| SaSignException::new(0, "missing nonce"))?;
        let signature = params
            .get(Self::SIGN)
            .filter(|value| !value.is_empty())
            .ok_or_else(|| SaSignException::new(0, "missing sign"))?;
        // Preserve Java validation order: timestamp, nonce persistence, signature.
        self.check_timestamp(timestamp)?;
        self.check_nonce(nonce)?;
        self.check_sign(params, signature)
    }
}
