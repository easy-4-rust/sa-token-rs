//! Sa-Token 初始化配置（对应 Java `SaTokenConfigure` + `application.yml`）。

use std::sync::Arc;

use sa_token::prelude::*;
use sa_token::sa_token_core::stp::StpInterface;

use super::StpInterfaceImpl;

/// 初始化 Sa-Token（Memory DAO + 权限扩展）。
pub fn init_sa_token() {
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
    SaManager::put_stp_logic(Arc::new(StpLogic::new("login")));
}
