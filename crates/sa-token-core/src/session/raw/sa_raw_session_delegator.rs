//! `SaRawSessionDelegator` —— 1:1 对应 Java `cn.dev33.satoken.session.raw.SaRawSessionDelegator`
//!
//! SaSession 读写工具类的委托类，将原始 session 操作委托给具体实现。

use std::sync::Arc;

use crate::exception::{SaResult, SaTokenException};
use crate::session::sa_session::SaSession;

/// raw session 类型（对应 Java `SaRawSessionDelegator` 中的 TYPE_* 常量）
pub struct SaRawSessionDelegator;

impl SaRawSessionDelegator {
    /// 默认 raw session 类型
    pub const TYPE_DEFAULT: &'static str = "default";

    /// 从 session id 读取一个 SaSession
    pub fn get_session(session_id: impl Into<String>) -> SaResult<Arc<SaSession>> {
        crate::session::raw::sa_raw_session_util::SaRawSessionUtil::get_session(session_id)
    }

    /// 写入一个 SaSession
    pub fn save_session(session: &SaSession, timeout: i64) -> SaResult<()> {
        let key = SaSession::splicing_key(session.id());
        let json =
            serde_json::to_string(session).map_err(|error| SaTokenException::JsonConvert {
                message: error.to_string(),
            })?;
        crate::sa_manager::SaManager::sa_token_dao().set(&key, &json, timeout)
    }

    /// 删除 session
    pub fn delete_session(session_id: impl Into<String>) -> SaResult<()> {
        let id = session_id.into();
        crate::sa_manager::SaManager::sa_token_dao().delete_session(&id)
    }
}
