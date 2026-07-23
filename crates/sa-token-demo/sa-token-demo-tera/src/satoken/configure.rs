//! Sa-Token 初始化（对应 Java `com.pj.satoken.SaTokenConfigure`）。

use std::sync::Arc;

use sa_token::prelude::*;
use sa_token::sa_token_core::stp::StpInterface;
use sa_token::sa_token_core::stp::stp_logic::StpLogic;

use super::stp_interface_impl::StpInterfaceImpl;

/// 初始化全局组件并返回默认 StpLogic（供 Tera 方言注册）。
///
/// 对应 Java Spring `@Configuration` 中的 Bean 注入。
pub fn init_sa_token() -> Arc<StpLogic> {
    SaManager::set_config(Arc::new(SaTokenConfig {
        token_name: "satoken".into(),
        timeout: 2_592_000,
        active_timeout: -1,
        is_concurrent: true,
        is_share: false,
        token_style: SaTokenStyle::Uuid,
        is_log: true,
        ..Default::default()
    }));
    SaManager::set_sa_token_dao(Arc::new(SaTokenDaoMemory::new()));
    SaManager::set_stp_interface(Arc::new(StpInterfaceImpl) as Arc<dyn StpInterface>);
    let logic = Arc::new(StpLogic::new("login"));
    SaManager::put_stp_logic(Arc::clone(&logic));
    logic
}
