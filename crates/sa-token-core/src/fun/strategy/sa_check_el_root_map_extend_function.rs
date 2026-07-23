//! `SaCheckELRootMapExtendFunction` —— 1:1 对应 Java `cn.dev33.satoken.fun.SaCheckELRootMapExtendFunction`

pub trait SaCheckELRootMapExtendFunction: Send + Sync + 'static {
    /// key/value: 添加到 EL root map 的键值对
    fn extend(&self, map: &mut std::collections::HashMap<String, serde_json::Value>);
}
