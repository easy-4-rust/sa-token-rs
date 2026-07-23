//! DAO 模块（对应 Java `cn.dev33.satoken.dao`）。

pub mod auto;
pub mod timed_cache;

// 1:1 对齐 Java `cn.dev33.satoken.dao`（根包）
pub mod async_sa_token_dao;
pub mod sa_token_dao;
pub mod sa_token_dao_default_impl;

pub use async_sa_token_dao::AsyncSaTokenDao;
pub use sa_token_dao::SaTokenDao;
