//! `SaJsonTemplateDefaultImpl` —— 1:1 对应 Java `cn.dev33.satoken.json.SaJsonTemplateDefaultImpl`
//!
//! 默认 JSON 模板实现（基于 serde_json）。

use super::sa_json_template::SaJsonTemplate;

/// 默认 JSON 模板实现（基于 serde_json）
pub struct SaJsonTemplateDefaultImpl;

impl SaJsonTemplate for SaJsonTemplateDefaultImpl {
    fn to_json(&self, value: &serde_json::Value) -> String {
        serde_json::to_string(value).unwrap_or_default()
    }

    fn parse_json(&self, json: &str) -> Option<serde_json::Value> {
        serde_json::from_str(json).ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn json_roundtrip() {
        let tpl = SaJsonTemplateDefaultImpl;
        let value = json!({"a": 1});
        let text = tpl.to_json(&value);
        assert_eq!(tpl.parse_json(&text), Some(value));
    }
}
