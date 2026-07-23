//! `sa-token-json-simd` —— simd-json 后端的 JSON 模板实现。
//!
//! 对应 Java `SaJsonTemplate` 的替代实现，底层使用 [simd-json](https://crates.io/crates/simd-json)
//! 提供的 SIMD 加速 JSON 解析（在现代 x86_64 平台典型 2-4x 加速）。
//!
//! 序列化沿用 `serde_json::to_string`（simd-json 暂无写入加速等价物），
//! 反序列化用 simd-json 优化路径，然后桥接到 `serde_json::Value`。

use sa_token_core::json::sa_json_template::SaJsonTemplate;

/// simd-json 后端的 JSON 模板实现
pub struct SaJsonTemplateSimdImpl;

impl SaJsonTemplate for SaJsonTemplateSimdImpl {
    fn to_json(&self, value: &serde_json::Value) -> String {
        serde_json::to_string(value).unwrap_or_default()
    }

    fn parse_json(&self, json: &str) -> Option<serde_json::Value> {
        // simd-json 需要 mutable bytes + in-place parse
        let mut bytes = json.as_bytes().to_vec();
        let value: simd_json::OwnedValue = simd_json::to_owned_value(&mut bytes).ok()?;
        Some(simd_to_serde(&value))
    }
}

fn simd_to_serde(v: &simd_json::OwnedValue) -> serde_json::Value {
    use simd_json::OwnedValue as O;
    use simd_json::StaticNode as S;
    match v {
        O::Static(s) => match s {
            S::Null => serde_json::Value::Null,
            S::Bool(b) => serde_json::Value::Bool(*b),
            S::I64(i) => serde_json::Value::Number(serde_json::Number::from(*i)),
            S::U64(u) => serde_json::Value::Number(serde_json::Number::from(*u)),
            S::F64(f) => serde_json::Number::from_f64(*f)
                .map(serde_json::Value::Number)
                .unwrap_or(serde_json::Value::Null),
        },
        O::String(s) => serde_json::Value::String(s.clone()),
        O::Array(arr) => serde_json::Value::Array(arr.iter().map(simd_to_serde).collect()),
        O::Object(obj) => {
            // halfbrown HashMap 通过引用迭代
            let mut map = serde_json::Map::with_capacity(obj.len());
            for (k, v) in obj.iter() {
                map.insert(k.to_string(), simd_to_serde(v));
            }
            serde_json::Value::Object(map)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn roundtrip_simple_object() {
        let tpl = SaJsonTemplateSimdImpl;
        let value = json!({"a": 1, "b": "hello", "c": [1, 2, 3]});
        let text = tpl.to_json(&value);
        let parsed = tpl.parse_json(&text).expect("parse should succeed");
        assert_eq!(parsed, value);
    }

    #[test]
    fn roundtrip_nested() {
        let tpl = SaJsonTemplateSimdImpl;
        let value = json!({
            "user": {"id": 10001, "roles": ["admin", "user"]},
            "meta": {"ts": 1_700_000_000_i64, "valid": true}
        });
        let text = tpl.to_json(&value);
        let parsed = tpl.parse_json(&text).expect("parse nested");
        assert_eq!(parsed, value);
    }

    #[test]
    fn invalid_json_returns_none() {
        let tpl = SaJsonTemplateSimdImpl;
        let result = tpl.parse_json("not json {");
        assert!(result.is_none());
    }
}
