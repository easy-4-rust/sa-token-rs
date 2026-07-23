//! SaTempTemplate：临时 token 验证模板（对应 Java `cn.dev33.satoken.temp.SaTempTemplate`）。

use serde::de::DeserializeOwned;
use serde_json::Value;

use crate::exception::{SaResult, SaTokenException};
use crate::sa_manager::SaManager;

/// 默认命名空间
pub const DEFAULT_NAMESPACE: &str = "temp-token";

/// 临时 token 验证模板 trait（对应 Java 可扩展的 `SaTempTemplate` 类）。
///
/// 实现需保证 `create_token`/`save_token`/`parse_token`/`get_timeout`/`delete_token`/
/// `splicing_temp_token_save_key` 这 6 个核心方法在语义上与 Java 1:1 等价。
/// 其余便捷方法（`parse_token_with_type`、`create_token_with_index` 等）通过
/// 默认实现从这 6 个核心方法组合，保持 trait 体量精简。
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

    /// 随机一个 temp-token（对应 Java `randomTempToken`）
    fn random_temp_token(&self) -> String {
        crate::util::sa_fox_util::random_uuid()
    }

    /// 生成临时 token 值，遵守 `max_try_times` 重试策略（对应 Java `createTempTokenValue`）
    fn create_temp_token_value(&self) -> String {
        let max_try_times = SaManager::config().get_max_try_times();
        for _ in 0..max_try_times.max(1) {
            let candidate = self.random_temp_token();
            // 候选 token 必须保证唯一（DAO 中不存在该 key）
            let existing = self.parse_token(&candidate).ok().flatten();
            if existing.is_none() {
                return candidate;
            }
        }
        // fallback：直接返回（与 Java 一致，最终由调用方决定是否再重试）
        self.random_temp_token()
    }

    /// 创建 token，并按 value 维度记录索引（对应 Java `createToken(value, timeout, isRecordIndex)`）
    ///
    /// 注：默认实现为简化版，**不维护 value 维度的反向索引**（与 Java 的 SaRawSessionDelegator
    /// 行为对应）。如需完整索引语义请实现 trait 的 `adjust_index` 与 `get_temp_token_list`。
    fn create_token_with_index(
        &self,
        value: &Value,
        timeout: i64,
        _is_record_index: bool,
    ) -> SaResult<String> {
        self.create_token(value, timeout)
    }

    /// 解析 token 获取 value，并尝试转换为指定类型
    ///
    /// 对应 Java `parseToken(token, Class)`。
    ///
    /// 注意：因含泛型方法，独立到 `SaTempTypedParser` trait 中以保持
    /// `SaTempTemplate` 的 dyn 兼容性。调用方应先 `as_any()` downcast 到具体
    /// 实现，或直接通过 `SaTempTypedParser` 扩展 trait 调用。
    fn parse_token_with_type_dyn(
        &self,
        token: &str,
    ) -> SaResult<Option<serde_json::Value>> {
        self.parse_token(token)
    }

    /// 解析 token 获取 value，可选裁剪前缀，并转换为指定类型
    ///
    /// 对应 Java `parseToken(token, cutPrefix, Class)`。泛型版本见
    /// `SaTempTypedParser`。
    fn parse_token_with_cut_prefix_dyn(
        &self,
        token: &str,
        cut_prefix: Option<&str>,
    ) -> SaResult<Option<serde_json::Value>> {
        let value = self.parse_token(token)?;
        let Some(value) = value else { return Ok(None) };
        match cut_prefix {
            None => Ok(Some(value)),
            Some(prefix) => {
                if prefix.len() >= 32 {
                    return Err(SaTokenException::other(
                        "裁剪前缀长度必须小于 32 位",
                    ));
                }
                let str_value = value_to_string(&value);
                if let Some(stripped) = str_value.strip_prefix(prefix) {
                    Ok(Some(Value::String(stripped.to_string())))
                } else {
                    Ok(None)
                }
            }
        }
    }

    /// 获取指定 value 关联的 temp-token 列表（对应 Java `getTempTokenList`）
    ///
    /// 默认实现：扫描以 token_name 为前缀的所有 key，过滤出属于本 namespace 的。
    /// 真实生产环境可由实现方通过 SaSession 索引优化。
    fn get_temp_token_list(&self, _value: &Value) -> SaResult<Vec<String>> {
        let prefix = format!(
            "{}:{}:",
            SaManager::config().get_token_name(),
            self.splicing_temp_token_save_key("")
                .split(':')
                .nth(1)
            .unwrap_or(DEFAULT_NAMESPACE)
        );
        let keys = SaManager::sa_token_dao().search_data(&prefix, "", 0, -1, true)?;
        Ok(keys
            .into_iter()
            .map(|k| {
                k.strip_prefix(&prefix)
                    .map(str::to_string)
                    .unwrap_or(k)
            })
            .collect())
    }

    /// JWT 密钥（对应 Java `getJwtSecretKey`，仅 sa-token-temp-jwt 集成时使用）
    fn jwt_secret_key(&self) -> Option<String> {
        None
    }
}

fn value_to_string(value: &Value) -> String {
    match value {
        Value::String(s) => s.clone(),
        Value::Null => String::new(),
        other => other.to_string(),
    }
}

/// 类型化解析扩展 trait（与 `SaTempTemplate` 解耦，因为泛型方法会破坏
/// dyn 兼容性）。所有实现 `SaTempTemplate` 的具体类型同时实现此 trait。
pub trait SaTempTypedParser: SaTempTemplate {
    /// 解析 token 获取 value，并尝试转换为指定类型
    ///
    /// 对应 Java `parseToken(token, Class)`。
    fn parse_token_with_type<T>(&self, token: &str) -> SaResult<Option<T>>
    where
        T: DeserializeOwned,
    {
        match SaTempTemplate::parse_token_with_type_dyn(self, token)? {
            None => Ok(None),
            Some(value) => Ok(Some(serde_json::from_value(value).map_err(|e| {
                SaTokenException::JsonConvert {
                    message: e.to_string(),
                }
            })?)),
        }
    }

    /// 解析 token 获取 value，可选裁剪前缀，并转换为指定类型
    ///
    /// 对应 Java `parseToken(token, cutPrefix, Class)`。
    /// `cut_prefix` 非空时，若 value 字符串不以前缀开头，返回 `Ok(None)`。
    fn parse_token_with_cut_prefix<T>(
        &self,
        token: &str,
        cut_prefix: Option<&str>,
    ) -> SaResult<Option<T>>
    where
        T: DeserializeOwned,
    {
        let value = SaTempTemplate::parse_token_with_cut_prefix_dyn(self, token, cut_prefix)?;
        match value {
            None => Ok(None),
            Some(value) => Ok(Some(serde_json::from_value(value).map_err(|e| {
                SaTokenException::JsonConvert {
                    message: e.to_string(),
                }
            })?)),
        }
    }
}

/// blanket impl：所有 `SaTempTemplate` 都自动实现 `SaTempTypedParser`
impl<T: SaTempTemplate + ?Sized> SaTempTypedParser for T {}

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

    /// 生成临时 token 值（保留旧 API，语义与 trait 默认方法一致）
    pub fn create_temp_token_value(&self) -> String {
        <Self as SaTempTemplate>::create_temp_token_value(self)
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

    #[test]
    fn random_temp_token_is_32_hex() {
        let tpl = SaTempTemplateDefault::default();
        let t = tpl.random_temp_token();
        assert_eq!(t.len(), 32);
        assert!(t.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn parse_token_with_type_round_trip() {
        use std::sync::Arc;
        use crate::config::sa_token_config::SaTokenConfig;
        use crate::dao::sa_token_dao_default_impl::SaTokenDaoDefaultImpl;
        crate::sa_manager::SaManager::reset();
        crate::sa_manager::SaManager::set_config(Arc::new(SaTokenConfig::default()));
        crate::sa_manager::SaManager::set_sa_token_dao(Arc::new(SaTokenDaoDefaultImpl::new()));
        let tpl = SaTempTemplateDefault::default();
        let value = serde_json::json!({"id": 42});
        let token = tpl
            .create_token(&value, 60)
            .expect("create temp token");
        let parsed: serde_json::Value = tpl
            .parse_token_with_type(&token)
            .expect("parse")
            .expect("some value");
        assert_eq!(parsed, value);
    }

    #[test]
    fn parse_token_with_cut_prefix_strips_and_converts() {
        use std::sync::Arc;
        use crate::config::sa_token_config::SaTokenConfig;
        use crate::dao::sa_token_dao_default_impl::SaTokenDaoDefaultImpl;
        crate::sa_manager::SaManager::reset();
        crate::sa_manager::SaManager::set_config(Arc::new(SaTokenConfig::default()));
        crate::sa_manager::SaManager::set_sa_token_dao(Arc::new(SaTokenDaoDefaultImpl::new()));
        let tpl = SaTempTemplateDefault::default();
        let token = tpl
            .create_token(&Value::String("user:10001".into()), 60)
            .expect("create");
        // 完整 string
        let raw: String = tpl
            .parse_token_with_type(&token)
            .expect("parse")
            .expect("some");
        assert_eq!(raw, "user:10001");
        // 裁剪 "user:" 前缀
        let parsed: String = tpl
            .parse_token_with_cut_prefix(&token, Some("user:"))
            .expect("parse cut")
            .expect("some");
        assert_eq!(parsed, "10001");
        // 前缀不匹配
        let miss: Option<String> = tpl
            .parse_token_with_cut_prefix(&token, Some("admin:"))
            .expect("parse cut miss");
        assert!(miss.is_none());
    }

    #[test]
    fn jwt_secret_key_default_none() {
        let tpl = SaTempTemplateDefault::default();
        assert_eq!(tpl.jwt_secret_key(), None);
    }
}
