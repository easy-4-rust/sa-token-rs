//! `sa-token-json-sonic` —— sonic-rs 后端的 JSON 模板实现。
//!
//! 对应 Java `SaJsonTemplate` 的替代实现，底层使用 [sonic-rs](https://crates.io/crates/sonic-rs)
//! （CloudWeGo sonic 的 Rust 移植版）。sonic-rs 在 SIMD 加速的基础上还提供
//! 更激进的零拷贝与 lazy evaluation 优化，在大量小 JSON 序列化场景下显著快于
//! serde_json。
//!
//! 序列化用 `sonic_rs::to_string`，反序列化用 `sonic_rs::from_str` 拿到
//! `sonic_rs::Value`，再通过其 `JsonValueTrait` 桥接到 `serde_json::Value`。
//!
//! `sonic_rs::Value` 0.3 不是 enum 而是用 `meta + data` 紧凑表示，因此用
//! trait 方法 `as_null` / `as_bool` / `as_i64` / `as_f64` / `as_str` / `as_array`
//! / `as_object` 来判别类型（任何不匹配则返回 Null）。

use sa_token_core::json::sa_json_template::SaJsonTemplate;
use sonic_rs::{JsonContainerTrait, JsonValueTrait};

/// sonic-rs 后端的 JSON 模板实现
pub struct SaJsonTemplateSonicImpl;

impl SaJsonTemplate for SaJsonTemplateSonicImpl {
    fn to_json(&self, value: &serde_json::Value) -> String {
        // sonic_rs 可以直接接受 serde_json::Value（实现了 Serialize）
        sonic_rs::to_string(value).unwrap_or_default()
    }

    fn parse_json(&self, json: &str) -> Option<serde_json::Value> {
        let v: sonic_rs::Value = sonic_rs::from_str(json).ok()?;
        Some(sonic_to_serde(&v))
    }
}

fn sonic_to_serde(v: &sonic_rs::Value) -> serde_json::Value {
    if v.is_null() {
        serde_json::Value::Null
    } else if let Some(b) = v.as_bool() {
        serde_json::Value::Bool(b)
    } else if let Some(i) = v.as_i64() {
        serde_json::Value::Number(serde_json::Number::from(i))
    } else if let Some(u) = v.as_u64() {
        serde_json::Value::Number(serde_json::Number::from(u))
    } else if let Some(f) = v.as_f64() {
        serde_json::Number::from_f64(f)
            .map(serde_json::Value::Number)
            .unwrap_or(serde_json::Value::Null)
    } else if let Some(s) = v.as_str() {
        serde_json::Value::String(s.to_string())
    } else if let Some(arr) = v.as_array() {
        serde_json::Value::Array(arr.iter().map(sonic_to_serde).collect())
    } else if let Some(obj) = v.as_object() {
        let mut map = serde_json::Map::with_capacity(obj.len());
        for (k, child) in obj.iter() {
            map.insert(k.to_string(), sonic_to_serde(child));
        }
        serde_json::Value::Object(map)
    } else {
        serde_json::Value::Null
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn roundtrip_simple_object() {
        let tpl = SaJsonTemplateSonicImpl;
        let value = json!({"a": 1, "b": "hello", "c": [1, 2, 3]});
        let text = tpl.to_json(&value);
        let parsed = tpl.parse_json(&text).expect("parse should succeed");
        assert_eq!(parsed, value);
    }

    #[test]
    fn roundtrip_nested_with_floats() {
        let tpl = SaJsonTemplateSonicImpl;
        let value = json!({
            "metrics": {"cpu": 0.85, "mem": 1024.0, "net": 0.0},
            "tags": ["prod", "v2"]
        });
        let text = tpl.to_json(&value);
        let parsed = tpl.parse_json(&text).expect("parse nested");
        assert_eq!(parsed, value);
    }

    #[test]
    fn invalid_json_returns_none() {
        let tpl = SaJsonTemplateSonicImpl;
        let result = tpl.parse_json("{invalid");
        assert!(result.is_none());
    }
}
