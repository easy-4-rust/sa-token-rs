//! SaTempUtil：临时 token 静态门面（对应 Java `cn.dev33.satoken.temp.SaTempUtil`）。

use serde_json::Value;

use crate::exception::SaResult;
use crate::sa_manager::SaManager;

/// 静态门面（实例通过 `SaManager::sa_temp_template()` 获取）
pub struct SaTempUtil;

impl SaTempUtil {
    /// 为指定 value 创建一个临时 token
    pub fn create_token(value: &Value, timeout: i64) -> SaResult<String> {
        SaManager::sa_temp_template().create_token(value, timeout)
    }

    /// 保存 token
    pub fn save_token(token: &str, value: &Value, timeout: i64) -> SaResult<()> {
        SaManager::sa_temp_template().save_token(token, value, timeout)
    }

    /// 解析 token 获取 value
    pub fn parse_token(token: &str) -> SaResult<Option<Value>> {
        SaManager::sa_temp_template().parse_token(token)
    }

    /// 获取剩余有效期（秒）
    pub fn get_timeout(token: &str) -> SaResult<i64> {
        SaManager::sa_temp_template().get_timeout(token)
    }

    /// 删除 token
    pub fn delete_token(token: &str) -> SaResult<()> {
        SaManager::sa_temp_template().delete_token(token)
    }
}
