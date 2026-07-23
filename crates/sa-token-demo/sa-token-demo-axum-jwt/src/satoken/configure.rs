//! JWT Demo 配置（对应 Java `StpLogicJwtForSimple`）。

use std::sync::Arc;

use sa_token::prelude::*;
use sa_token_jwt::StpLogicJwtForSimple;

/// JWT 密钥（演示用，生产请外置）。
pub const JWT_SECRET: &str = "asdhfjkasdhfjkashdfjkashdfkjahsdjkfhaskldjf";

/// 初始化 Sa-Token + JWT Simple 模式辅助。
pub fn init_sa_token() -> StpLogicJwtForSimple {
    SaManager::set_config(Arc::new(SaTokenConfig {
        token_name: "satoken".into(),
        timeout: 2_592_000,
        is_concurrent: true,
        is_share: false,
        is_log: true,
        ..Default::default()
    }));
    SaManager::set_sa_token_dao(Arc::new(SaTokenDaoMemory::new()));
    SaManager::put_stp_logic(Arc::new(StpLogic::new("login")));
    StpLogicJwtForSimple::new("login", JWT_SECRET)
}
