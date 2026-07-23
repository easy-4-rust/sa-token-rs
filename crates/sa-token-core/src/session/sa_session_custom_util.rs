//! `SaSessionCustomUtil` —— 1:1 对应 Java `cn.dev33.satoken.session.SaSessionCustomUtil`
//!
//! 自定义 SaSession 工具类，按 sessionId 快速读写数据。

use std::sync::Arc;

use crate::exception::SaResult;
use crate::sa_manager::SaManager;

/// 自定义 SaSession 工具（对应 Java `SaSessionCustomUtil`）
pub struct SaSessionCustomUtil;

impl SaSessionCustomUtil {
    /// 根据 sessionId 获取一个 SaSession（不存在则新建）
    pub fn get_session_by_id(
        session_id: impl Into<String>,
    ) -> SaResult<Arc<crate::session::sa_session::SaSession>> {
        let id = session_id.into();
        let dao = SaManager::sa_token_dao();
        if let Some(session) = dao.get_session(&id)? {
            return Ok(Arc::new(session));
        }
        let session = crate::session::sa_session::SaSession::new(&id);
        dao.set_session(&session, -1)?;
        Ok(Arc::new(session))
    }

    /// 删除指定 session
    pub fn delete_session_by_id(session_id: impl Into<String>) -> SaResult<()> {
        let id = session_id.into();
        let dao = SaManager::sa_token_dao();
        dao.delete_session(&id)
    }
}
