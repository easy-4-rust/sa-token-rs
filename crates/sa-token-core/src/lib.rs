//! Sa-Token-Rs Core: 核心类型和扩展点
//!
//! 对应 Java Sa-Token 的 `sa-token-core` 模块。
//!
//! ## 目录命名约定
//!
//! 本项目采用 **snake_case 模块路径 + PascalCase 类型** 的 Rust 标准命名：
//!
//! | Java 类名 | Rust 文件路径 |
//! | --- | --- |
//! | `cn.dev33.satoken.SaManager` | `src/sa_manager.rs` |
//! | `cn.dev33.satoken.context.SaTokenContext` | `src/context/sa_token_context.rs` |
//! | `cn.dev33.satoken.context.mock.SaRequestForMock` | `src/context/mock/sa_request_for_mock.rs` |
//! | `cn.dev33.satoken.secure.BCrypt` | `src/secure/bcrypt.rs` |
//!
//! **类型名**仍然 PascalCase 以严格对齐 Java 类名，但目录遵守 Rust 默认 snake_case 习惯。

pub mod annotation;
pub mod application;
pub mod config;
pub mod context;
pub mod dao;
pub mod error;
pub mod exception;
pub mod filter;
pub mod fun;
pub mod http;
pub mod http_auth;
pub mod json;
pub mod listener;
pub mod log;
pub mod model;
pub mod plugin;
pub mod router;
pub mod runtime;
pub mod sa_manager;
pub mod same;
pub mod secure;
pub mod serializer;
pub mod session;
pub mod stp;
pub mod strategy;
pub mod temp;
pub mod util;
