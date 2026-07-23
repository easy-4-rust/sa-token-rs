//! Serialization ports and concrete adapters.

// ---------- 子模块声明 ----------
pub mod r#impl;
pub mod sa_serializer_template;

// ---------- re-exports ----------
pub use r#impl::{
    SaSerializerTemplateForJdk, SaSerializerTemplateForJdkUseBase64,
    SaSerializerTemplateForJdkUseHex, SaSerializerTemplateForJdkUseIso88591,
    SaSerializerTemplateForJson,
};
pub use sa_serializer_template::SaSerializerTemplate;
