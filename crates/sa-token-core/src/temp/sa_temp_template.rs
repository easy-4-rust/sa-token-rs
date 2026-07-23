//! SaTempTemplate：临时 token 验证模板（对应 Java `cn.dev33.satoken.temp.SaTempTemplate`）。

use serde_json::Value;

use crate::exception::SaResult;
use crate::sa_manager::SaManager;

/// 默认命名空间
pub const DEFAULT_NAMESPACE: &str = "temp-token";

/// 临时 token 验证模板 trait（对应 Java 可扩展的 `SaTempTemplate` 类）。
pub trait SaTempTemplate: Send + Sync {
    /// 创建临时 token
    fn create_token(&self, value: &Value, timeout: i64) -> SaResult<String>;

    /// 保存 token → value 映射
    fn save_token(&self, token: &str, value: &Value, timeout: i64) -> SaResult<()>;

    /// 解析 token 获取 value
    fn parse_token(&self, token: &str) -> SaResult<Option<Value>>;

    /// 获取剩余有效期（秒）；-1 永久；-2 无效
    fn get_timeout(&self, token: &str) -> SaResult<i64>;

    /// 删除 token
    fn delete_token(&self, token: &str) -> SaResult<()>;

    /// 拼接持久化 key
    fn splicing_temp_token_save_key(&self, token: &str) -> String;
}

/// 默认 DAO 持久化实现（对应 Java 基类 `SaTempTemplate` 的默认行为）。
pub struct SaTempTemplateDefault {
    pub namespace: String,
}

impl Default for SaTempTemplateDefault {
    fn default() -> Self {
        Self::new(DEFAULT_NAMESPACE)
    }
}

impl SaTempTemplateDefault {
    /// 创建指定命名空间的模板
    pub fn new(namespace: impl Into<String>) -> Self {
        let namespace = namespace.into();
        if namespace.is_empty() {
            panic!("namespace 不能为空");
        }
        Self { namespace }
    }

    /// 生成临时 token 值
    pub fn create_temp_token_value(&self) -> String {
        crate::util::sa_fox_util::random_uuid()
    }

    fn get_value(&self, token: &str) -> SaResult<Option<Value>> {
        let key = self.splicing_temp_token_save_key(token);
        SaManager::sa_token_dao().get_object(&key)
    }

    fn delete_token_inner(&self, token: &str) -> SaResult<()> {
        let key = self.splicing_temp_token_save_key(token);
        SaManager::sa_token_dao().delete_object(&key)
    }

    fn get_timeout_inner(&self, token: &str) -> SaResult<i64> {
        let key = self.splicing_temp_token_save_key(token);
        SaManager::sa_token_dao().get_object_timeout(&key)
    }
}

impl SaTempTemplate for SaTempTemplateDefault {
    fn create_token(&self, value: &Value, timeout: i64) -> SaResult<String> {
        let token = self.create_temp_token_value();
        self.save_token(&token, value, timeout)?;
        Ok(token)
    }

    fn save_token(&self, token: &str, value: &Value, timeout: i64) -> SaResult<()> {
        let key = self.splicing_temp_token_save_key(token);
        SaManager::sa_token_dao().set_object(&key, value, timeout)
    }

    fn parse_token(&self, token: &str) -> SaResult<Option<Value>> {
        self.get_value(token)
    }

    fn get_timeout(&self, token: &str) -> SaResult<i64> {
        self.get_timeout_inner(token)
    }

    fn delete_token(&self, token: &str) -> SaResult<()> {
        self.delete_token_inner(token)
    }

    fn splicing_temp_token_save_key(&self, token: &str) -> String {
        format!(
            "{}:{}:{}",
            SaManager::config().get_token_name(),
            self.namespace,
            token
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_namespace_constant() {
        assert_eq!(DEFAULT_NAMESPACE, "temp-token");
    }

    #[test]
    fn splice_key_includes_namespace_and_token_name() {
        let tpl = SaTempTemplateDefault::new("my-temp");
        let key = tpl.splicing_temp_token_save_key("TOKEN");
        assert!(key.contains("my-temp"));
        assert!(key.contains("TOKEN"));
    }

    #[test]
    fn create_temp_token_value_is_uuid_format() {
        let tpl = SaTempTemplateDefault::default();
        let v = tpl.create_temp_token_value();
        assert_eq!(v.len(), 32);
        assert!(v.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    #[should_panic(expected = "namespace 不能为空")]
    fn empty_namespace_panics() {
        let _ = SaTempTemplateDefault::new("");
    }
}
