//! JSON 模板接口（对应 Java `cn.dev33.satoken.json.SaJsonTemplate`）。

/// JSON 序列化/反序列化 trait
pub trait SaJsonTemplate: Send + Sync + 'static {
    /// 将对象序列化为 JSON 字符串
    fn to_json(&self, value: &serde_json::Value) -> String;

    /// 将 JSON 字符串反序列化为对象
    fn parse_json(&self, json: &str) -> Option<serde_json::Value>;
}
