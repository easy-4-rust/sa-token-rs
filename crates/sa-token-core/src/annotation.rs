//! 注解模块（对应 Java `cn.dev33.satoken.annotation`）。
//!
//! 注解宏在独立的 `sa-token-derive` crate 中实现。
//! 本模块保留与 Java 1:1 对齐的运行时常量、Meta 类型与 handler 接口。

// 1:1 对齐 Java `cn.dev33.satoken.annotation` 包
pub mod sa_check_disable;
pub mod sa_check_http_basic;
pub mod sa_check_http_digest;
pub mod sa_check_login;
pub mod sa_check_or;
pub mod sa_check_permission;
pub mod sa_check_role;
pub mod sa_check_safe;
pub mod sa_ignore;
pub mod sa_mode;

// 1:1 对齐 Java `cn.dev33.satoken.annotation.handler` 子包
pub mod handler;
