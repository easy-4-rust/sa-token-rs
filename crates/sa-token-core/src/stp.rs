//! STP 模块（对应 Java `cn.dev33.satoken.stp`）。

pub mod async_stp_logic;
pub mod async_stp_util;
pub mod parameter;
pub mod sa_login_config;
pub mod sa_login_model;
pub mod sa_token_info;
pub mod stp_interface;
pub mod stp_interface_default_impl;
pub mod stp_logic;
pub mod stp_util;

// Stable, intentionally selected facade exports.
pub use async_stp_logic::AsyncStpLogic;
pub use async_stp_util::AsyncStpUtil;
pub use sa_login_config::SaLoginConfig;
pub use sa_login_model::SaLoginModel;
pub use sa_token_info::SaTokenInfo;
pub use stp_interface::StpInterface;
pub use stp_interface_default_impl::StpInterfaceDefaultImpl;
pub use stp_util::STYPE;
pub use stp_util::StpUtil as SaStpUtil;
pub use stp_util::StpUtilType as _StpUtilType;

pub(crate) mod shared {
    use base64::Engine;

    use crate::config::sa_token_config::{SaTokenConfig, SaTokenStyle};

    pub(crate) fn create_token_value(config: &SaTokenConfig) -> String {
        match config.token_style {
            SaTokenStyle::Uuid => uuid::Uuid::new_v4().to_string(),
            SaTokenStyle::SimpleUuid => uuid::Uuid::new_v4().to_string().replace('-', ""),
            SaTokenStyle::Random32 => crate::util::sa_fox_util::random_string(32),
            SaTokenStyle::Random64 => crate::util::sa_fox_util::random_string(64),
            SaTokenStyle::Random128 => crate::util::sa_fox_util::random_string(128),
            SaTokenStyle::Base64 => {
                let random = crate::util::sa_fox_util::random_string(32);
                base64::engine::general_purpose::STANDARD.encode(random.as_bytes())
            }
            SaTokenStyle::Jwt => uuid::Uuid::new_v4().to_string(),
            SaTokenStyle::Tik => {
                format!(
                    "gr_SwoIN0MC1ewxHX_vfCW3BothWDZMMtx__{}",
                    uuid::Uuid::new_v4().simple()
                )
            }
        }
    }

    pub(crate) fn token_key(token_name: &str, login_type: &str, token_value: &str) -> String {
        format!("{token_name}:{login_type}:token:{token_value}")
    }

    pub(crate) fn session_key(token_name: &str, login_type: &str, login_id: &str) -> String {
        format!("{token_name}:{login_type}:session:{login_id}")
    }

    pub(crate) fn token_session_key(
        token_name: &str,
        login_type: &str,
        token_value: &str,
    ) -> String {
        format!("{token_name}:{login_type}:token-session:{token_value}")
    }

    pub(crate) fn last_active_key(token_name: &str, login_type: &str, token_value: &str) -> String {
        format!("{token_name}:{login_type}:last-active:{token_value}")
    }

    pub(crate) fn disable_key(
        token_name: &str,
        login_type: &str,
        login_id: &str,
        service: &str,
    ) -> String {
        format!("{token_name}:{login_type}:disable:{service}:{login_id}")
    }

    pub(crate) fn safe_key(
        token_name: &str,
        login_type: &str,
        token_value: &str,
        service: &str,
    ) -> String {
        format!("{token_name}:{login_type}:safe:{service}:{token_value}")
    }

    pub(crate) fn switch_key(login_type: &str) -> String {
        format!("SWITCH_TO_SAVE_KEY_{login_type}")
    }
}
