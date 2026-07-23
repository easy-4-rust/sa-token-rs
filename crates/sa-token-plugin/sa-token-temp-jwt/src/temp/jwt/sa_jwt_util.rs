//! JWT helpers for temporary tokens (Java `cn.dev33.satoken.temp.jwt.SaJwtUtil`).

use jsonwebtoken::errors::ErrorKind;
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation, decode, encode};
use sa_token_core::dao::sa_token_dao::{NEVER_EXPIRE, NOT_VALUE_EXPIRE};
use sa_token_core::exception::{SaResult, SaTokenException};
use sa_token_core::secure::sa_secure_util::SaSecureUtil;
use sa_token_core::util::sa_fox_util::now_timestamp_millis;
use serde_json::{Map, Value};

use super::error::sa_temp_jwt_error_code::SaTempJwtErrorCode;

/// Temp-jwt utility facade.
pub struct SaJwtUtil;

impl SaJwtUtil {
    /// Claim key holding the stored value.
    pub const KEY_VALUE: &'static str = "value_";
    /// Claim key holding expiry timestamp (milliseconds) or `-1`.
    pub const KEY_EFF: &'static str = "eff";
    /// Never-expire marker shared with DAO semantics.
    pub const NEVER_EXPIRE: i64 = NEVER_EXPIRE;

    /// Creates a JWT temp-token for the given value and timeout (seconds).
    pub fn create_token(value: &Value, timeout: i64, secret: &str) -> SaResult<String> {
        let eff = if timeout == NEVER_EXPIRE {
            NEVER_EXPIRE
        } else {
            timeout
                .saturating_mul(1000)
                .saturating_add(now_timestamp_millis())
        };
        let mut payload = Map::new();
        payload.insert(Self::KEY_VALUE.into(), value.clone());
        payload.insert(Self::KEY_EFF.into(), Value::from(eff));
        Self::generate_token(&payload, secret)
    }

    /// Parses a JWT temp-token and returns its claims map.
    pub fn parse_token(jwt_token: &str, secret: &str) -> SaResult<Map<String, Value>> {
        let secret = Self::signing_key(secret)?;
        if jwt_token.is_empty() {
            return Err(SaTokenException::with_code(
                SaTempJwtErrorCode::CODE_30303,
                "JWT string must not be empty",
            ));
        }
        let mut validation = Validation::new(Algorithm::HS256);
        validation.required_spec_claims.clear();
        validation.validate_exp = false;
        decode::<Map<String, Value>>(
            jwt_token,
            &DecodingKey::from_secret(secret.as_bytes()),
            &validation,
        )
        .map(|data| data.claims)
        .map_err(|error| {
            let code = if matches!(error.kind(), ErrorKind::InvalidSignature) {
                SaTempJwtErrorCode::CODE_30303
            } else {
                SaTempJwtErrorCode::CODE_30303
            };
            SaTokenException::with_code(code, error.to_string())
        })
    }

    /// Parses a JWT temp-token and returns the embedded value.
    pub fn get_value(jwt_token: &str, secret: &str) -> SaResult<Value> {
        let claims = Self::parse_token(jwt_token, secret)?;
        let eff = claims
            .get(Self::KEY_EFF)
            .and_then(Value::as_i64)
            .unwrap_or(0);
        if eff != NEVER_EXPIRE && eff < now_timestamp_millis() {
            return Err(SaTokenException::with_code(
                SaTempJwtErrorCode::CODE_30303,
                format!("token 已超时，无法解析：{jwt_token}"),
            ));
        }
        claims
            .get(Self::KEY_VALUE)
            .cloned()
            .ok_or_else(|| {
                SaTokenException::with_code(
                    SaTempJwtErrorCode::CODE_30303,
                    format!("token 缺少 value 载荷：{jwt_token}"),
                )
            })
    }

    /// Returns remaining lifetime in seconds; `-1` forever, `-2` invalid/expired.
    pub fn get_timeout(jwt_token: &str, secret: &str) -> SaResult<i64> {
        let claims = Self::parse_token(jwt_token, secret)?;
        let eff = claims
            .get(Self::KEY_EFF)
            .and_then(Value::as_i64)
            .unwrap_or(0);
        if eff == NEVER_EXPIRE {
            return Ok(NEVER_EXPIRE);
        }
        let now = now_timestamp_millis();
        if eff < now {
            return Ok(NOT_VALUE_EXPIRE);
        }
        Ok((eff - now) / 1000)
    }

    fn generate_token(payload: &Map<String, Value>, secret: &str) -> SaResult<String> {
        let secret = Self::signing_key(secret)?;
        encode(
            &Header::new(Algorithm::HS256),
            &Value::Object(payload.clone()),
            &EncodingKey::from_secret(secret.as_bytes()),
        )
        .map_err(|error| {
            SaTokenException::with_code(SaTempJwtErrorCode::CODE_30303, error.to_string())
        })
    }

    fn signing_key(secret: &str) -> SaResult<String> {
        if sa_token_core::util::sa_fox_util::is_empty(secret) {
            return Err(SaTokenException::with_code(
                SaTempJwtErrorCode::CODE_30301,
                "请配置：jwtSecretKey",
            ));
        }
        Ok(SaSecureUtil::md5(secret))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn create_and_parse_round_trip() {
        let secret = "temp-secret";
        let value = json!("payload-10001");
        let token = SaJwtUtil::create_token(&value, 60, secret).expect("create");
        assert!(!token.is_empty());
        let parsed = SaJwtUtil::get_value(&token, secret).expect("parse");
        assert_eq!(parsed, value);
        let timeout = SaJwtUtil::get_timeout(&token, secret).expect("timeout");
        assert!(timeout > 0 && timeout <= 60);
    }

    #[test]
    fn missing_secret_uses_detailed_code() {
        let err = SaJwtUtil::create_token(&json!("x"), 60, "")
            .expect_err("missing secret");
        assert_eq!(err.code(), SaTempJwtErrorCode::CODE_30301);
    }
}
