//! `SaTokenDaoByObjectFollowString` —— 1:1 对应 Java `cn.dev33.satoken.dao.SaTokenDaoByObjectFollowString`
//!
//! Object 读写跟随 String 读写的默认实现辅助。

use crate::dao::sa_token_dao::SaTokenDao;
use crate::exception::SaResult;
use crate::serializer::SaSerializerTemplate;

/// Object 层跟随 String 层的 DAO 扩展（对应 Java 接口默认方法）。
pub trait SaTokenDaoByObjectFollowString: SaTokenDao {}

/// 通过 String 层读取 Object
pub fn get_object(
    dao: &impl SaTokenDao,
    serializer: &impl SaSerializerTemplate,
    key: &str,
) -> SaResult<Option<serde_json::Value>> {
    serializer.string_to_object(dao.get(key)?.as_deref())
}

/// 通过 String 层写入 Object
pub fn set_object(
    dao: &impl SaTokenDao,
    serializer: &impl SaSerializerTemplate,
    key: &str,
    object: &serde_json::Value,
    timeout: i64,
) -> SaResult<()> {
    let json = serializer
        .object_to_string(Some(object))?
        .unwrap_or_default();
    dao.set(key, &json, timeout)
}

/// 通过 String 层更新 Object
pub fn update_object(
    dao: &impl SaTokenDao,
    serializer: &impl SaSerializerTemplate,
    key: &str,
    object: &serde_json::Value,
) -> SaResult<()> {
    let json = serializer
        .object_to_string(Some(object))?
        .unwrap_or_default();
    dao.update(key, &json)
}

/// 通过 String 层删除 Object
pub fn delete_object(dao: &impl SaTokenDao, key: &str) -> SaResult<()> {
    dao.delete(key)
}

/// 通过 String 层获取 Object 剩余存活时间
pub fn get_object_timeout(dao: &impl SaTokenDao, key: &str) -> SaResult<i64> {
    dao.get_timeout(key)
}

/// 通过 String 层更新 Object 剩余存活时间
pub fn update_object_timeout(dao: &impl SaTokenDao, key: &str, timeout: i64) -> SaResult<()> {
    dao.update_timeout(key, timeout)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dao::sa_token_dao_default_impl::SaTokenDaoDefaultImpl;
    use crate::serializer::r#impl::SaSerializerTemplateForJson;

    #[test]
    fn object_follow_string_roundtrip() {
        let dao = SaTokenDaoDefaultImpl::new();
        let serializer = SaSerializerTemplateForJson;
        let value = serde_json::json!({"name": "sa-token"});
        set_object(&dao, &serializer, "obj", &value, -1).expect("set object");
        assert_eq!(
            get_object(&dao, &serializer, "obj").expect("get object"),
            Some(value)
        );
    }
}
