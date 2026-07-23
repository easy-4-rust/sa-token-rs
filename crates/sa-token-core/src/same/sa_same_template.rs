//! Same-Token 模板方法（对应 Java `cn.dev33.satoken.same.SaSameTemplate`）。

use std::sync::Arc;

use crate::config::sa_token_config::SaTokenConfig;
use crate::context::sa_holder::SaHolder;
use crate::dao::sa_token_dao::SaTokenDao;
use crate::exception::{SaResult, SaTokenException};
use crate::sa_manager::SaManager;

/// Same-Token 模板方法类
///
/// 对应 Java `SaSameTemplate`。
pub struct SaSameTemplate {
    /// 当前 Same-Token 在请求头中的字段名
    pub same_token: &'static str,
    dao: Option<Arc<dyn SaTokenDao>>,
    config: Option<Arc<SaTokenConfig>>,
}

impl Default for SaSameTemplate {
    fn default() -> Self {
        Self::new()
    }
}

impl SaSameTemplate {
    /// 默认字段名（对应 Java `SAME_TOKEN = "SA-SAME-TOKEN"`）
    pub const SAME_TOKEN: &'static str = "SA-SAME-TOKEN";

    /// 创建默认实例
    pub fn new() -> Self {
        Self {
            same_token: Self::SAME_TOKEN,
            dao: None,
            config: None,
        }
    }

    /// Creates an isolated Same-Token template.
    pub fn with_runtime(dao: Arc<dyn SaTokenDao>, config: Arc<SaTokenConfig>) -> Self {
        Self {
            same_token: Self::SAME_TOKEN,
            dao: Some(dao),
            config: Some(config),
        }
    }

    fn dao(&self) -> Arc<dyn SaTokenDao> {
        self.dao.clone().unwrap_or_else(SaManager::sa_token_dao)
    }

    fn config(&self) -> Arc<SaTokenConfig> {
        self.config.clone().unwrap_or_else(SaManager::config)
    }

    // -------------------- 获取 & 校验 --------------------

    /// 获取当前 Same-Token, 如果不存在则立即创建并返回
    ///
    /// 对应 Java `SaSameTemplate.getToken()`。
    pub fn get_token(&self) -> SaResult<String> {
        let current = self.get_token_nh()?;
        if current.is_empty() {
            // 自刷新——注意高并发下不严格可用
            self.refresh_token()
        } else {
            Ok(current)
        }
    }

    /// 判断一个 Same-Token 是否有效
    pub fn is_valid(&self, token: &str) -> SaResult<bool> {
        if token.is_empty() {
            return Ok(false);
        }
        Ok(token == self.get_token()? || token == self.get_past_token_nh()?)
    }

    /// 校验一个 Same-Token 是否有效（无效则抛出异常）
    pub fn check_token(&self, token: &str) -> SaResult<()> {
        if !self.is_valid(token)? {
            return Err(SaTokenException::with_code(
                10301,
                format!("无效Same-Token：{token}"),
            ));
        }
        Ok(())
    }

    /// 校验当前 Request 上下文提供的 Same-Token 是否有效
    pub fn check_current_request_token(&self) -> SaResult<()> {
        let req = SaHolder::get_request();
        let header = req.get_header(self.same_token).unwrap_or_default();
        self.check_token(&header)
    }

    /// 刷新一次 Same-Token（注意集群环境中不要多个服务重复调用）
    pub fn refresh_token(&self) -> SaResult<String> {
        // 1. 将当前 Same-Token 写入 Past-Same-Token
        let current = self.get_token_nh()?;
        if !current.is_empty() {
            self.save_past_token(&current, self.get_token_timeout()?)?;
        }
        // 2. 刷新当前 Same-Token
        let new_token = self.create_token();
        self.save_token(&new_token)?;
        Ok(new_token)
    }

    // -------------------- 保存 --------------------

    /// 保存 Same-Token
    pub fn save_token(&self, token: &str) -> SaResult<()> {
        if token.is_empty() {
            return Ok(());
        }
        let timeout = self.config().get_same_token_timeout();
        self.dao()
            .set(&self.splicing_token_save_key(), token, timeout)
    }

    /// 保存 Past-Same-Token
    pub fn save_past_token(&self, token: &str, timeout: i64) -> SaResult<()> {
        if token.is_empty() {
            return Ok(());
        }
        self.dao()
            .set(&self.splicing_past_token_save_key(), token, timeout)
    }

    // -------------------- 获取原始 --------------------

    /// 获取 Same-Token，不做任何处理
    pub fn get_token_nh(&self) -> SaResult<String> {
        Ok(self
            .dao()
            .get(&self.splicing_token_save_key())?
            .unwrap_or_default())
    }

    /// 获取 Past-Same-Token，不做任何处理
    pub fn get_past_token_nh(&self) -> SaResult<String> {
        Ok(self
            .dao()
            .get(&self.splicing_past_token_save_key())?
            .unwrap_or_default())
    }

    /// 获取 Same-Token 的剩余有效期（秒）
    pub fn get_token_timeout(&self) -> SaResult<i64> {
        self.dao().get_timeout(&self.splicing_token_save_key())
    }

    // -------------------- 创建 --------------------

    /// 创建一个 Same-Token（64 字符随机字符串）
    pub fn create_token(&self) -> String {
        crate::util::sa_fox_util::random_string(64)
    }

    // -------------------- 拼接 key --------------------

    /// Same-Token 存储 key
    pub fn splicing_token_save_key(&self) -> String {
        format!("{}:var:same-token", self.config().get_token_name())
    }

    /// Past-Same-Token 存储 key
    pub fn splicing_past_token_save_key(&self) -> String {
        format!("{}:var:past-same-token", self.config().get_token_name())
    }
}

pub const SAME_TOKEN: &str = SaSameTemplate::SAME_TOKEN;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_token_length_is_64() {
        let tpl = SaSameTemplate::new();
        let token = tpl.create_token();
        assert_eq!(token.len(), 64);
    }

    #[test]
    fn default_field_name() {
        assert_eq!(SaSameTemplate::SAME_TOKEN, "SA-SAME-TOKEN");
    }

    #[test]
    fn splicing_keys() {
        let tpl = SaSameTemplate::new();
        assert!(tpl.splicing_token_save_key().ends_with(":var:same-token"));
        assert!(
            tpl.splicing_past_token_save_key()
                .ends_with(":var:past-same-token")
        );
    }

    #[test]
    fn empty_token_invalid() {
        let tpl = SaSameTemplate::new();
        assert!(!tpl.is_valid("").expect("validation should succeed"));
    }

    #[test]
    fn isolated_runtime_rotates_current_and_past_tokens_with_java_error_code() {
        use crate::dao::sa_token_dao_default_impl::SaTokenDaoDefaultImpl;

        let config = SaTokenConfig {
            token_name: "isolated".into(),
            same_token_timeout: 120,
            ..Default::default()
        };
        let template =
            SaSameTemplate::with_runtime(Arc::new(SaTokenDaoDefaultImpl::new()), Arc::new(config));
        let first = template.refresh_token().expect("first token");
        let second = template.refresh_token().expect("second token");
        assert_ne!(first, second);
        assert!(template.is_valid(&first).expect("past token remains valid"));
        assert!(template.is_valid(&second).expect("current token is valid"));
        assert_eq!(
            template
                .check_token("invalid")
                .expect_err("invalid token")
                .code(),
            10301
        );
        assert_eq!(
            template.splicing_token_save_key(),
            "isolated:var:same-token"
        );
    }
}
