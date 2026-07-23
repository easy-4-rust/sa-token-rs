//! `SaRawSessionUtil` —— 1:1 对应 Java `cn.dev33.satoken.session.raw.SaRawSessionUtil`
//!
//! SaSession 读写工具类（按 sessionId 直接读写）。

use std::sync::Arc;

use crate::exception::SaResult;
use crate::sa_manager::SaManager;
use crate::session::sa_session::SaSession;

/// raw session 工具类（对应 Java `SaRawSessionUtil`）
pub struct SaRawSessionUtil;

impl SaRawSessionUtil {
    /// 读取一个 raw session
    pub fn get_session(session_id: impl Into<String>) -> SaResult<Arc<SaSession>> {
        let id = session_id.into();
        let dao = SaManager::sa_token_dao();
        if let Some(session) = dao.get_session(&id)? {
            return Ok(Arc::new(session));
        }
        let session = SaSession::new(&id);
        dao.set_session(&session, -1)?;
        Ok(Arc::new(session))
    }

    /// 写入 raw session
    pub fn save_session(session: &SaSession, timeout: i64) -> SaResult<()> {
        SaManager::sa_token_dao().set_session(session, timeout)
    }

    /// 删除 raw session
    pub fn delete_session(session_id: impl Into<String>) -> SaResult<()> {
        SaManager::sa_token_dao().delete_session(&session_id.into())
    }
}
