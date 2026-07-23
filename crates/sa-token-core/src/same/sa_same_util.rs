//! Same-Token 全局静态门面（对应 Java `cn.dev33.satoken.same.SaSameUtil`）。

use crate::exception::SaResult;
use crate::sa_manager::SaManager;

/// Same-Token 全局静态门面
pub struct SaSameUtil;

impl SaSameUtil {
    /// 获取当前 Same-Token（不存在则创建）
    pub fn get_token() -> SaResult<String> {
        SaManager::sa_same_template().get_token()
    }

    /// 判断 Same-Token 是否有效
    pub fn is_valid(token: &str) -> SaResult<bool> {
        SaManager::sa_same_template().is_valid(token)
    }

    /// 校验一个 Same-Token
    pub fn check_token(token: &str) -> SaResult<()> {
        SaManager::sa_same_template().check_token(token)
    }

    /// 校验当前请求上下文的 Same-Token
    pub fn check_current_request_token() -> SaResult<()> {
        SaManager::sa_same_template().check_current_request_token()
    }

    /// 刷新一次 Same-Token
    pub fn refresh_token() -> SaResult<String> {
        SaManager::sa_same_template().refresh_token()
    }

    /// Returns the current token without creating one.
    pub fn get_token_nh() -> SaResult<String> {
        SaManager::sa_same_template().get_token_nh()
    }

    /// Returns the previous token without creating one.
    pub fn get_past_token_nh() -> SaResult<String> {
        SaManager::sa_same_template().get_past_token_nh()
    }
}
