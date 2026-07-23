//! Sa-Token-Rs: 轻量级权限认证框架
//!
//! 对应 Java Sa-Token，提供登录认证、权限认证、会话管理等能力。
//!
//! # 快速开始
//!
//! ```rust
//! use sa_token::prelude::*;
//!
//! // 初始化
//! SaManager::set_config(Arc::new(SaTokenConfig::default()));
//! SaManager::set_sa_token_dao(Arc::new(SaTokenDaoMemory::new()));
//! SaTokenContextMockUtil::set_mock_context();
//! SaManager::put_stp_logic(Arc::new(StpLogic::new("login")));
//!
//! // 登录
//! StpUtil::login("10001").unwrap();
//! assert!(StpUtil::is_login().unwrap());
//! assert_eq!(StpUtil::get_login_id().unwrap(), "10001");
//!
//! // 登出
//! StpUtil::logout().unwrap();
//! assert!(!StpUtil::is_login().unwrap());
//! ```

// Re-export 核心模块
pub use sa_token_context_mock;
pub use sa_token_core;
pub use sa_token_dao_memory;

#[cfg(feature = "redis")]
pub use sa_token_dao_redis;

#[cfg(feature = "actix-web")]
pub use sa_token_web_actix;

#[cfg(feature = "salvo")]
pub use sa_token_web_salvo;

#[cfg(feature = "apikey")]
pub use sa_token_apikey;

#[cfg(feature = "serializer-features")]
pub use sa_token_serializer_features;

#[cfg(feature = "jwt")]
pub use sa_token_jwt;

#[cfg(feature = "temp-jwt")]
pub use sa_token_temp_jwt;

#[cfg(feature = "quick-login")]
pub use sa_token_quick_login;

#[cfg(feature = "sign")]
pub use sa_token_sign;

// Re-export derive macros
pub use sa_token_derive::*;

/// 预导入模块
pub mod prelude {
    #[cfg(feature = "apikey")]
    pub use sa_token_apikey::{
        ApiKeyModel, SaApiKeyConfig, SaApiKeyManager, SaApiKeyTemplate, SaApiKeyUtil,
    };
    pub use sa_token_context_mock::SaTokenContextMockUtil;
    pub use sa_token_core::config::sa_token_config::{SaTokenConfig, SaTokenStyle};
    pub use sa_token_core::context::sa_holder::SaHolder;
    pub use sa_token_core::exception::{SaResult, SaTokenException};
    pub use sa_token_core::runtime::{AsyncSaTokenRuntime, SaTokenRuntime};
    pub use sa_token_core::sa_manager::SaManager;
    pub use sa_token_core::session::sa_session::SaSession;
    pub use sa_token_core::session::sa_terminal_info::SaTerminalInfo;
    pub use sa_token_core::stp::parameter::sa_login_parameter::SaLoginParameter;
    pub use sa_token_core::stp::parameter::sa_logout_parameter::SaLogoutParameter;
    pub use sa_token_core::stp::sa_token_info::SaTokenInfo;
    pub use sa_token_core::stp::stp_logic::StpLogic;
    pub use sa_token_core::stp::stp_util::StpUtil;
    pub use sa_token_core::stp::{AsyncStpLogic, AsyncStpUtil};
    pub use sa_token_dao_memory::SaTokenDaoMemory;
    #[cfg(feature = "jwt")]
    pub use sa_token_jwt::{
        SaJwtErrorCode, SaJwtException, SaJwtTemplate, SaJwtUtil, StpLogicJwtForMixin,
        StpLogicJwtForSimple, StpLogicJwtForStateless,
    };
    #[cfg(feature = "temp-jwt")]
    pub use sa_token_temp_jwt::{
        SaJwtUtil as SaTempJwtUtil, SaTempJwtErrorCode, SaTempTemplateForJwt,
        SaTokenPluginForTempForJwt,
    };
    #[cfg(feature = "quick-login")]
    pub use sa_token_quick_login::{
        DoLoginHandleFunction, SaQuickConfig, SaQuickController, SaQuickInject, SaQuickManager,
        SaQuickRegister,
    };
    #[cfg(feature = "serializer-features")]
    pub use sa_token_serializer_features::{
        SaSerializerForBase64UseCustomCharacters, SaSerializerForBase64UseEmoji,
        SaSerializerForBase64UsePeriodicTable, SaSerializerForBase64UseSpecialSymbols,
        SaSerializerForBase64UseTianGan,
    };
    #[cfg(feature = "sign")]
    pub use sa_token_sign::{
        SaCheckSign, SaSignConfig, SaSignErrorCode, SaSignException, SaSignManager, SaSignMany,
        SaSignTemplate, SaSignUtil,
    };

    // Re-export derive macros
    pub use sa_token_derive::{
        sa_check_disable, sa_check_login, sa_check_permission, sa_check_role, sa_check_safe,
        sa_ignore,
    };

    // 便捷 re-export
    pub use std::sync::Arc;
}
