//! `SaTokenDaoByStringFollowObject` —— 1:1 对应 Java `cn.dev33.satoken.dao.SaTokenDaoByStringFollowObject`
//!
//! String 读写跟随 Object 读写的默认实现辅助。

use crate::dao::sa_token_dao::SaTokenDao;
use crate::exception::SaResult;
use crate::serializer::SaSerializerTemplate;

/// String 层跟随 Object 层的 DAO 扩展（对应 Java 接口默认方法）。
pub trait SaTokenDaoByStringFollowObject: SaTokenDao {}

/// 通过 Object 层读取 String
pub fn get(
    dao: &impl SaTokenDao,
    serializer: &impl SaSerializerTemplate,
    key: &str,
) -> SaResult<Option<String>> {
    match dao.get_object(key)? {
        None => Ok(None),
        Some(serde_json::Value::String(s)) => Ok(Some(s)),
        Some(value) => Ok(serializer.object_to_string(Some(&value))?),
    }
}

/// 通过 Object 层写入 String
pub fn set(
    dao: &impl SaTokenDao,
    key: &str,
    value: &str,
    timeout: i64,
) -> SaResult<()> {
    dao.set_object(key, &serde_json::Value::String(value.to_string()), timeout)
}

/// 通过 Object 层更新 String
pub fn update(dao: &impl SaTokenDao, key: &str, value: &str) -> SaResult<()> {
    dao.update_object(key, &serde_json::Value::String(value.to_string()))
}

/// 通过 Object 层删除 String
pub fn delete(dao: &impl SaTokenDao, key: &str) -> SaResult<()> {
    dao.delete_object(key)
}

/// 通过 Object 层获取 String 剩余存活时间
pub fn get_timeout(dao: &impl SaTokenDao, key: &str) -> SaResult<i64> {
    dao.get_object_timeout(key)
}

/// 通过 Object 层更新 String 剩余存活时间
pub fn update_timeout(dao: &impl SaTokenDao, key: &str, timeout: i64) -> SaResult<()> {
    dao.update_object_timeout(key, timeout)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dao::sa_token_dao_default_impl::SaTokenDaoDefaultImpl;
    use crate::serializer::r#impl::SaSerializerTemplateForJson;

    #[test]
    fn string_follow_object_roundtrip() {
        let dao = SaTokenDaoDefaultImpl::new();
        let serializer = SaSerializerTemplateForJson;
        set(&dao, "k", "v", -1).expect("set string");
        assert_eq!(
            get(&dao, &serializer, "k").expect("get string"),
            Some("v".to_string())
        );
    }
}
