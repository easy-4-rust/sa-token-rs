//! 存储接口（对应 Java `cn.dev33.satoken.dao.SaTokenDao`）。
use crate::session::sa_session::SaSession;

/// 永不过期
pub const NEVER_EXPIRE: i64 = -1;

/// 值不存在时的过期时间标记
pub const NOT_VALUE_EXPIRE: i64 = -2;

/// Sa-Token 持久化接口
///
/// 对应 Java `SaTokenDao`，定义了 Token、Session 等数据的存储操作。
pub trait SaTokenDao: Send + Sync + 'static {
    // ==================== 字符串读写 ====================

    /// 获取值
    fn get(&self, key: &str) -> Option<String>;

    /// 设置值（带超时，单位：秒）
    fn set(&self, key: &str, value: &str, timeout: i64);

    /// 更新值（不修改超时时间）
    fn update(&self, key: &str, value: &str);

    /// 删除值
    fn delete(&self, key: &str);

    /// 获取值的剩余超时时间（秒）
    fn get_timeout(&self, key: &str) -> i64;

    /// 修改值的超时时间（秒）
    fn update_timeout(&self, key: &str, timeout: i64);

    // ==================== 对象读写 ====================

    /// 获取 Object
    fn get_object(&self, key: &str) -> Option<serde_json::Value>;

    /// 存储 Object（带超时）
    fn set_object(&self, key: &str, value: &serde_json::Value, timeout: i64);

    /// 更新 Object（不修改超时时间）
    fn update_object(&self, key: &str, value: &serde_json::Value);

    /// 删除 Object
    fn delete_object(&self, key: &str);

    /// 获取 Object 的剩余超时时间
    fn get_object_timeout(&self, key: &str) -> i64;

    /// 修改 Object 的超时时间
    fn update_object_timeout(&self, key: &str, timeout: i64);

    // ==================== SaSession 读写 ====================

    /// 获取 Session
    fn get_session(&self, session_id: &str) -> Option<SaSession>;

    /// 存储 Session（带超时）
    fn set_session(&self, session: &SaSession, timeout: i64);

    /// 更新 Session（不修改超时时间）
    fn update_session(&self, session: &SaSession);

    /// 删除 Session
    fn delete_session(&self, session_id: &str);

    /// 获取 Session 的剩余超时时间
    fn get_session_timeout(&self, session_id: &str) -> i64;

    /// 修改 Session 的超时时间
    fn update_session_timeout(&self, session_id: &str, timeout: i64);

    // ==================== 搜索 ====================

    /// 搜索数据
    fn search_data(
        &self,
        prefix: &str,
        keyword: &str,
        start: i64,
        size: i64,
        sort_type: bool,
    ) -> Vec<String>;
}
