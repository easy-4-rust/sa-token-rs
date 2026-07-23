//! 构建 AsyncStpUtil（对应 Java Sa-Token 配置）。

use std::sync::Arc;

use sa_token::prelude::{AsyncSaTokenRuntime, AsyncStpUtil, SaTokenConfig, SaTokenDaoMemory};
use sa_token_core::context::sa_token_context_default_impl::SaTokenContextDefaultImpl;
use sa_token_core::stp::StpInterface;

use super::StpInterfaceImpl;

/// 创建带权限扩展的 AsyncStpUtil。
pub fn build_stp_util() -> AsyncStpUtil {
    let runtime = AsyncSaTokenRuntime::new(
        Arc::new(SaTokenConfig {
            token_name: "satoken".into(),
            timeout: 2_592_000,
            active_timeout: -1,
            is_concurrent: true,
            is_share: false,
            is_log: true,
            ..Default::default()
        }),
        Arc::new(SaTokenDaoMemory::new()),
        Arc::new(SaTokenContextDefaultImpl),
    )
    .with_stp_interface(Arc::new(StpInterfaceImpl) as Arc<dyn StpInterface>);

    AsyncStpUtil::new("login", Arc::new(runtime))
}
