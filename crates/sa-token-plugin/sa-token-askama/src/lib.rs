//! Sa-Token Askama 方言插件（对应 Java `sa-token-thymeleaf`）。
//!
//! 技术映射：Thymeleaf → **Askama**；Jackson → serde。

pub mod askama;

pub use askama::dialect::{
    DEFAULT_DIALECT_NAME, DEFAULT_PRECEDENCE, SaTokenDialect, SaTokenTagProcessor, TagPredicate,
};
