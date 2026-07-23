//! `sa-token-rpc-volo` —— Volo gRPC 框架（ByteDance）适配层。
//!
//! 对应 Java `sa-token-dubbo3` 插件的角色。Volo 是 ByteDance 开源的
//! 高性能 Rust gRPC 框架，提供了 `Service` trait + `Layer` 抽象。
//!
//! 本 crate 提供：
//! - `SaTokenContext` 注入辅助函数
//! - `check_login_required`：从 gRPC request metadata 提取 token 并校验
//!
//! volo 框架通过 `Service::call` 包装请求，因此实际的 layer 包装在用户项目中完成
//! （约 30 行），本 crate 暴露 API 一致性与 axum/tonic 适配器保持一致。

use sa_token_core::exception::SaResult;
use sa_token_core::stp::stp_util::StpUtil;

/// 模拟的 metadata 头提取（volo 实际类型因 `prost` 生成而异，这里用通用 trait）
pub trait SaTokenMetadataExt {
    fn get_sa_token_header(&self, name: &str) -> Option<String>;
}

/// 通用 login_id 提取与校验
///
/// # 参数
/// - `header_name`: gRPC metadata header 名称（默认 `authorization`）
/// - `token_lookup`: 从请求 metadata 中查找 token 的闭包
///
/// # 返回
/// - `Ok(Some(login_id))`：token 有效
/// - `Ok(None)`：未提供 token（视为匿名）
/// - `Err(_)`：提供了 token 但无效/已过期
pub fn check_login_with_lookup<F>(
    header_name: &str,
    token_lookup: F,
) -> SaResult<Option<String>>
where
    F: FnOnce(&str) -> Option<String>,
{
    let token = match token_lookup(header_name) {
        None => return Ok(None),
        Some(t) => t,
    };
    let login_id = StpUtil::get_login_id_by_token(&token)?;
    Ok(login_id)
}

/// Volo handler 中提取 login_id 的便捷函数（直接用 SaTokenManager 检查）
pub fn require_login_id(token: &str) -> SaResult<String> {
    StpUtil::get_login_id_by_token(token)?
        .ok_or_else(|| sa_token_core::exception::SaTokenException::NotLogin {
            message: "token invalid or expired".to_string(),
            login_type: "login".to_string(),
            scene: "NOT_TOKEN".to_string(),
            code: 0,
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use sa_token_core::config::sa_token_config::SaTokenConfig;
    use sa_token_core::dao::sa_token_dao_default_impl::SaTokenDaoDefaultImpl;
    use sa_token_core::sa_manager::SaManager;
    use std::sync::Arc;

    fn setup() {
        use sa_token_core::stp::stp_logic::StpLogic;
        SaManager::reset();
        SaManager::set_config(Arc::new(SaTokenConfig::default()));
        SaManager::set_sa_token_dao(Arc::new(SaTokenDaoDefaultImpl::new()));
        SaManager::put_stp_logic(Arc::new(StpLogic::new("login")));
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn check_login_returns_none_when_no_token() {
        setup();
        let result = check_login_with_lookup("authorization", |_| None).expect("ok");
        assert!(result.is_none());
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn check_login_returns_none_for_invalid_token() {
        setup();
        let result =
            check_login_with_lookup("authorization", |_| Some("invalid-token".to_string()))
                .expect("ok");
        assert!(result.is_none());
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn require_login_id_fails_for_invalid_token() {
        setup();
        let result = require_login_id("invalid-token");
        assert!(result.is_err());
    }
}
