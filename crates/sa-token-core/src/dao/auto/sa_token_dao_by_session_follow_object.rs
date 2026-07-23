//! `SaTokenDaoBySessionFollowObject` —— 1:1 对应 Java `cn.dev33.satoken.dao.SaTokenDaoBySessionFollowObject`
//!
//! Session 读写跟随 Object 读写的默认实现辅助。

use crate::dao::sa_token_dao::SaTokenDao;
use crate::exception::{SaResult, SaTokenException};
use crate::session::sa_session::SaSession;

/// Session 层跟随 Object 层的 DAO 扩展（对应 Java 接口默认方法）。
pub trait SaTokenDaoBySessionFollowObject: SaTokenDao {}

/// 通过 Object 层读取 Session
pub fn get_session(dao: &impl SaTokenDao, session_id: &str) -> SaResult<Option<SaSession>> {
    match dao.get_object(session_id)? {
        None => Ok(None),
        Some(value) => serde_json::from_value(value).map(Some).map_err(|error| {
            SaTokenException::JsonConvert {
                message: error.to_string(),
            }
        }),
    }
}

/// 通过 Object 层写入 Session
pub fn set_session(dao: &impl SaTokenDao, session: &SaSession, timeout: i64) -> SaResult<()> {
    let value = serde_json::to_value(session).map_err(|error| SaTokenException::JsonConvert {
        message: error.to_string(),
    })?;
    dao.set_object(session.id(), &value, timeout)
}

/// 通过 Object 层更新 Session
pub fn update_session(dao: &impl SaTokenDao, session: &SaSession) -> SaResult<()> {
    let value = serde_json::to_value(session).map_err(|error| SaTokenException::JsonConvert {
        message: error.to_string(),
    })?;
    dao.update_object(session.id(), &value)
}

/// 通过 Object 层删除 Session
pub fn delete_session(dao: &impl SaTokenDao, session_id: &str) -> SaResult<()> {
    dao.delete_object(session_id)
}

/// 通过 Object 层获取 Session 剩余存活时间
pub fn get_session_timeout(dao: &impl SaTokenDao, session_id: &str) -> SaResult<i64> {
    dao.get_object_timeout(session_id)
}

/// 通过 Object 层更新 Session 剩余存活时间
pub fn update_session_timeout(
    dao: &impl SaTokenDao,
    session_id: &str,
    timeout: i64,
) -> SaResult<()> {
    dao.update_object_timeout(session_id, timeout)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dao::sa_token_dao_default_impl::SaTokenDaoDefaultImpl;

    #[test]
    fn session_follow_object_roundtrip() {
        let dao = SaTokenDaoDefaultImpl::new();
        let session = SaSession::new("sess-1");
        set_session(&dao, &session, -1).expect("set session");
        let loaded = get_session(&dao, "sess-1")
            .expect("get session")
            .expect("session exists");
        assert_eq!(loaded.id(), "sess-1");
    }
}
