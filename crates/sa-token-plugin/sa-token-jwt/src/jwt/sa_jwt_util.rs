//! Stateless convenience facade over [`SaJwtTemplate`].

use std::collections::HashMap;

use serde_json::{Map, Value};

use super::{SaJwtResult, SaJwtTemplate};

/// Java-compatible static-style JWT utility.
pub struct SaJwtUtil;

impl SaJwtUtil {
    pub const LOGIN_TYPE: &'static str = SaJwtTemplate::LOGIN_TYPE;
    pub const LOGIN_ID: &'static str = SaJwtTemplate::LOGIN_ID;
    pub const DEVICE_TYPE: &'static str = SaJwtTemplate::DEVICE_TYPE;
    pub const EFF: &'static str = SaJwtTemplate::EFF;
    pub const RN_STR: &'static str = SaJwtTemplate::RN_STR;
    pub const NEVER_EXPIRE: i64 = SaJwtTemplate::NEVER_EXPIRE;
    pub const NOT_VALUE_EXPIRE: i64 = SaJwtTemplate::NOT_VALUE_EXPIRE;

    pub fn create_token(
        login_type: &str,
        login_id: Value,
        extra_data: HashMap<String, Value>,
        secret: &str,
    ) -> SaJwtResult<String> {
        SaJwtTemplate.create_token(login_type, login_id, extra_data, secret)
    }

    pub fn create_token_full(
        login_type: &str,
        login_id: Value,
        device_type: &str,
        timeout: i64,
        extra_data: HashMap<String, Value>,
        secret: &str,
    ) -> SaJwtResult<String> {
        SaJwtTemplate.create_token_full(
            login_type,
            login_id,
            device_type,
            timeout,
            extra_data,
            secret,
        )
    }

    pub fn create_token_from_map(
        payload: &Map<String, Value>,
        secret: &str,
    ) -> SaJwtResult<String> {
        SaJwtTemplate.generate_token(payload, secret)
    }

    pub fn parse_token(
        token: &str,
        login_type: &str,
        secret: &str,
        check_timeout: bool,
    ) -> SaJwtResult<Map<String, Value>> {
        SaJwtTemplate.parse_token(token, login_type, secret, check_timeout)
    }

    pub fn get_payloads(
        token: &str,
        login_type: &str,
        secret: &str,
    ) -> SaJwtResult<Map<String, Value>> {
        SaJwtTemplate.get_payloads(token, login_type, secret)
    }

    pub fn get_payloads_not_check(
        token: &str,
        login_type: &str,
        secret: &str,
    ) -> SaJwtResult<Map<String, Value>> {
        SaJwtTemplate.get_payloads_not_check(token, login_type, secret)
    }

    pub fn get_login_id(token: &str, login_type: &str, secret: &str) -> SaJwtResult<Value> {
        SaJwtTemplate.get_login_id(token, login_type, secret)
    }

    pub fn get_login_id_or_none(token: &str, login_type: &str, secret: &str) -> Option<Value> {
        SaJwtTemplate.get_login_id_or_none(token, login_type, secret)
    }

    pub fn get_timeout(token: &str, login_type: &str, secret: &str) -> i64 {
        SaJwtTemplate.get_timeout(token, login_type, secret)
    }
}
