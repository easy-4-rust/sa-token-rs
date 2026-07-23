//! 监听器模块（对应 Java `cn.dev33.satoken.listener`）。

pub mod sa_token_event_center;
pub mod sa_token_listener;
pub mod sa_token_listener_for_log;
pub mod sa_token_listener_for_simple;

pub use sa_token_listener::{
    LISTENERS, SaTokenListener as SaListenerTrait, listeners, register_listener,
};
