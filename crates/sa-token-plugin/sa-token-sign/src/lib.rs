//! API request-signature plugin migrated from Java `sa-token-sign`.

pub mod plugin;
pub mod sign;

pub use plugin::SaTokenPluginForSign;
pub use sign::{
    SaCheckSign, SaCheckSignHandler, SaSignConfig, SaSignErrorCode, SaSignException, SaSignManager,
    SaSignMany, SaSignManyConfigWrapper, SaSignTemplate, SaSignUtil,
};
