//! Sa-Token 配置与权限扩展（对应 Java `com.pj.satoken`）。

pub mod configure;
pub mod stp_interface_impl;

pub use configure::init_sa_token;
pub use stp_interface_impl::StpInterfaceImpl;
