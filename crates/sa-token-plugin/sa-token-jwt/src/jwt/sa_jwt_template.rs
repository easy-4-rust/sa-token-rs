//! HS256 JWT implementation matching Java `SaJwtTemplate` claim semantics.

use std::collections::HashMap;

use jsonwebtoken::errors::ErrorKind;
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation, decode, encode};
use sa_token_core::dao::sa_token_dao::{NEVER_EXPIRE, NOT_VALUE_EXPIRE};
use sa_token_core::util::sa_fox_util::{now_timestamp_millis, random_string};
use serde_json::{Map, Value};

use super::error::SaJwtErrorCode;
use super::exception::{SaJwtException, SaJwtResult};

/// JWT template using the Java claim names and millisecond `eff` timestamp.
#[derive(Debug, Default, Clone, Copy)]
pub struct SaJwtTemplate;

impl SaJwtTemplate {
    pub const LOGIN_TYPE: &'static str = "loginType";
    pub const LOGIN_ID: &'static str = "loginId";
    pub const DEVICE_TYPE: &'static str = "deviceType";
    pub const EFF: &'static str = "eff";
    pub const RN_STR: &'static str = "rnStr";
    pub const NEVER_EXPIRE: i64 = NEVER_EXPIRE;
    pub const NOT_VALUE_EXPIRE: i64 = NOT_VALUE_EXPIRE;

    /// Creates a simple-mode token without an expiry claim.
    pub fn create_token(
        &self,
        login_type: &str,
        login_id: Value,
        extra_data: HashMap<String, Value>,
        secret: &str,
    ) -> SaJwtResult<String> {
        if login_id.is_null() {
            return Err(SaJwtException::new(
                SaJwtErrorCode::CODE_30206,
                "login id must not be null",
            ));
        }
        let mut payload = Map::new();
        payload.insert(Self::LOGIN_TYPE.into(), Value::String(login_type.into()));
        payload.insert(Self::LOGIN_ID.into(), login_id);
        payload.insert(Self::RN_STR.into(), Value::String(random_string(32)));
        payload.extend(extra_data);
        self.generate_token(&payload, secret)
    }

    /// Creates a mixin/stateless token with device and millisecond expiry claims.
    pub fn create_token_full(
        &self,
        login_type: &str,
        login_id: Value,
        device_type: &str,
        timeout: i64,
        extra_data: HashMap<String, Value>,
        secret: &str,
    ) -> SaJwtResult<String> {
        if login_id.is_null() {
            return Err(SaJwtException::new(
                SaJwtErrorCode::CODE_30206,
                "login id must not be null",
            ));
        }
        let expiry = if timeout == NEVER_EXPIRE {
            NEVER_EXPIRE
        } else {
            now_timestamp_millis().saturating_add(timeout.saturating_mul(1000))
        };
        let mut payload = Map::new();
        payload.insert(Self::LOGIN_TYPE.into(), Value::String(login_type.into()));
        payload.insert(Self::LOGIN_ID.into(), login_id);
        payload.insert(Self::DEVICE_TYPE.into(), Value::String(device_type.into()));
        payload.insert(Self::EFF.into(), Value::from(expiry));
        payload.insert(Self::RN_STR.into(), Value::String(random_string(32)));
        payload.extend(extra_data);
        self.generate_token(&payload, secret)
    }

    /// Signs an arbitrary payload using HS256 and the raw secret bytes.
    pub fn generate_token(
        &self,
        payload: &Map<String, Value>,
        secret: &str,
    ) -> SaJwtResult<String> {
        let secret = Self::required_secret(secret)?;
        encode(
            &Header::new(Algorithm::HS256),
            payload,
            &EncodingKey::from_secret(secret.as_bytes()),
        )
        .map_err(|error| SaJwtException::new(SaJwtErrorCode::CODE_30201, error.to_string()))
    }

    /// Parses and validates signature, login type, and optionally expiry.
    pub fn parse_token(
        &self,
        token: &str,
        login_type: &str,
        secret: &str,
        check_timeout: bool,
    ) -> SaJwtResult<Map<String, Value>> {
        let secret = Self::required_secret(secret)?;
        if token.is_empty() {
            return Err(SaJwtException::new(
                SaJwtErrorCode::CODE_30201,
                "JWT string must not be empty",
            ));
        }
        let mut validation = Validation::new(Algorithm::HS256);
        validation.required_spec_claims.clear();
        validation.validate_exp = false;
        let payload = decode::<Map<String, Value>>(
            token,
            &DecodingKey::from_secret(secret.as_bytes()),
            &validation,
        )
        .map_err(|error| {
            let code = if matches!(error.kind(), ErrorKind::InvalidSignature) {
                SaJwtErrorCode::CODE_30202
            } else {
                SaJwtErrorCode::CODE_30201
            };
            SaJwtException::new(code, error.to_string())
        })?
        .claims;
        if payload.get(Self::LOGIN_TYPE).and_then(Value::as_str) != Some(login_type) {
            return Err(SaJwtException::new(
                SaJwtErrorCode::CODE_30203,
                "JWT login type is invalid",
            ));
        }
        if check_timeout {
            let expiry = payload.get(Self::EFF).and_then(Value::as_i64).unwrap_or(0);
            if expiry != NEVER_EXPIRE && expiry < now_timestamp_millis() {
                return Err(SaJwtException::new(
                    SaJwtErrorCode::CODE_30204,
                    "JWT has expired",
                ));
            }
        }
        Ok(payload)
    }

    pub fn get_payloads(
        &self,
        token: &str,
        login_type: &str,
        secret: &str,
    ) -> SaJwtResult<Map<String, Value>> {
        self.parse_token(token, login_type, secret, true)
    }

    pub fn get_payloads_not_check(
        &self,
        token: &str,
        login_type: &str,
        secret: &str,
    ) -> SaJwtResult<Map<String, Value>> {
        self.parse_token(token, login_type, secret, false)
    }

    pub fn get_login_id(&self, token: &str, login_type: &str, secret: &str) -> SaJwtResult<Value> {
        self.get_payloads(token, login_type, secret)?
            .remove(Self::LOGIN_ID)
            .ok_or_else(|| {
                SaJwtException::new(SaJwtErrorCode::CODE_30206, "JWT login id is absent")
            })
    }

    pub fn get_login_id_or_none(
        &self,
        token: &str,
        login_type: &str,
        secret: &str,
    ) -> Option<Value> {
        self.get_login_id(token, login_type, secret).ok()
    }

    pub fn get_timeout(&self, token: &str, login_type: &str, secret: &str) -> i64 {
        let Ok(payload) = self.parse_token(token, login_type, secret, false) else {
            return NOT_VALUE_EXPIRE;
        };
        let Some(expiry) = payload.get(Self::EFF).and_then(Value::as_i64) else {
            return NOT_VALUE_EXPIRE;
        };
        if expiry == NEVER_EXPIRE {
            return NEVER_EXPIRE;
        }
        let remaining = expiry.saturating_sub(now_timestamp_millis());
        if remaining <= 0 {
            NOT_VALUE_EXPIRE
        } else {
            remaining / 1000
        }
    }

    fn required_secret(secret: &str) -> SaJwtResult<&str> {
        SaJwtException::require_non_empty(
            Some(secret),
            "JWT secret must be configured",
            SaJwtErrorCode::CODE_30205,
        )
    }
}
