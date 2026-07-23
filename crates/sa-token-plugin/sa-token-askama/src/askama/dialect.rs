//! Thymeleaf 方言 → Askama 辅助类型（对应 Java `...thymeleaf.dialect`）。

pub mod sa_token_dialect;
pub mod sa_token_tag_processor;

pub use sa_token_dialect::{DEFAULT_DIALECT_NAME, DEFAULT_PRECEDENCE, SaTokenDialect};
pub use sa_token_tag_processor::{SaTokenTagProcessor, TagPredicate};
