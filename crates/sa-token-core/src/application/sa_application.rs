//! SaApplication：全局应用对象（对应 Java `cn.dev33.satoken.application.SaApplication`）。
//!
//! 提供应用级别的 KV 存储能力（使用 SaTokenDao 持久化）。

use std::sync::OnceLock;

use serde_json::Value;

use super::sa_get_value_interface::SaGetValueInterface;
use super::sa_set_value_interface::SaSetValueInterface;
use crate::dao::sa_token_dao::NEVER_EXPIRE;
use crate::exception::SaResult;
use crate::sa_manager::SaManager;

/// 全局应用对象
///
/// 对应 Java `SaApplication.defaultInstance`。
pub struct SaApplication;

impl SaApplication {
    /// 默认实例
    pub fn default_instance() -> &'static Self {
        static INSTANCE: OnceLock<SaApplication> = OnceLock::new();
        INSTANCE.get_or_init(|| SaApplication)
    }

    /// 拼接存储 key
    pub fn splicing_data_key(key: &str) -> String {
        format!("{}:var:{}", SaManager::config().get_token_name(), key)
    }

    /// Returns all application keys without the internal prefix.
    ///
    /// # Errors
    ///
    /// Returns a DAO search failure.
    pub fn keys(&self) -> SaResult<Vec<String>> {
        let prefix = Self::splicing_data_key("");
        SaManager::sa_token_dao()
            .search_data(&prefix, "", 0, -1, true)
            .map(|keys| {
                keys.into_iter()
                    .filter_map(|key| key.strip_prefix(&prefix).map(str::to_owned))
                    .collect()
            })
    }

    /// Deletes all application-scoped values.
    ///
    /// # Errors
    ///
    /// Returns the first DAO search or delete failure.
    pub fn clear(&self) -> SaResult<()> {
        for key in self.keys()? {
            self.delete(&key)?;
        }
        Ok(())
    }
}

impl SaGetValueInterface for SaApplication {
    fn get(&self, key: &str) -> SaResult<Option<Value>> {
        SaManager::sa_token_dao().get_object(&Self::splicing_data_key(key))
    }
}

impl SaSetValueInterface for SaApplication {
    fn set(&self, key: &str, value: Value) -> SaResult<&Self> {
        SaManager::sa_token_dao().set_object(
            &Self::splicing_data_key(key),
            &value,
            NEVER_EXPIRE,
        )?;
        Ok(self)
    }

    fn set_with_ttl(&self, key: &str, value: Value, ttl: i64) -> SaResult<&Self> {
        SaManager::sa_token_dao().set_object(&Self::splicing_data_key(key), &value, ttl)?;
        Ok(self)
    }

    fn delete(&self, key: &str) -> SaResult<&Self> {
        SaManager::sa_token_dao().delete_object(&Self::splicing_data_key(key))?;
        Ok(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Mutex, MutexGuard};

    static TEST_LOCK: Mutex<()> = Mutex::new(());

    fn setup() -> MutexGuard<'static, ()> {
        let guard = TEST_LOCK
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        use crate::config::sa_token_config::SaTokenConfig;
        use std::sync::Arc;
        let cfg = Arc::new(SaTokenConfig::default());
        SaManager::set_config(cfg);
        SaManager::set_sa_token_dao(Arc::new(
            crate::dao::sa_token_dao_default_impl::SaTokenDaoDefaultImpl::new(),
        ));
        guard
    }

    #[test]
    fn splicing_key_format() {
        let _guard = setup();
        assert_eq!(SaApplication::splicing_data_key("foo"), "satoken:var:foo");
    }

    #[test]
    fn set_and_get() {
        let _guard = setup();
        let app = SaApplication::default_instance();
        app.set("k1", Value::String("v1".into()))
            .expect("application set should succeed");
        let v = app.get("k1").expect("application get should succeed");
        assert_eq!(v, Some(Value::String("v1".into())));
    }

    #[test]
    fn java_application_contract_covers_typed_reads_lazy_values_keys_and_clear() {
        let _guard = setup();
        let app = SaApplication::default_instance();
        app.clear().expect("clean initial application state");

        app.set("age", Value::String("18".into()))
            .expect("store age");
        assert_eq!(app.get_i32("age").expect("integer conversion"), 18);
        assert_eq!(app.get_i64("age").expect("long conversion"), 18);
        assert_eq!(app.get_f32("age").expect("float conversion"), 18.0);
        assert_eq!(app.get_f64("age").expect("double conversion"), 18.0);
        assert_eq!(
            app.get_string("age").expect("string conversion").as_deref(),
            Some("18")
        );
        assert_eq!(
            app.get_with_default("missing", 20).expect("default value"),
            20
        );
        assert_eq!(
            app.get_or_insert_with("lazy", || Value::String("23".into()))
                .expect("lazy insertion"),
            Value::String("23".into())
        );
        app.set_by_null("stable", Value::String("18".into()))
            .expect("first conditional write");
        app.set_by_null("stable", Value::String("20".into()))
            .expect("second conditional write");
        assert_eq!(app.get_i32("stable").expect("stable value"), 18);

        let keys = app.keys().expect("list application keys");
        assert_eq!(keys, ["age", "lazy", "stable"]);
        app.clear().expect("clear application keys");
        assert!(app.keys().expect("empty application keys").is_empty());
    }

    #[test]
    fn application_info_cuts_only_configured_non_root_prefix() {
        use crate::application::application_info::ApplicationInfo;

        ApplicationInfo::set_route_prefix("/api");
        assert_eq!(ApplicationInfo::cut_path_prefix("/api/users"), "/users");
        assert_eq!(ApplicationInfo::cut_path_prefix("/other"), "/other");
        ApplicationInfo::set_route_prefix("/");
        assert_eq!(ApplicationInfo::cut_path_prefix("/users"), "/users");
    }
}
