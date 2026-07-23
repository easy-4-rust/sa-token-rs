//! Sa-Token Tera 方言插件（对应 Java `sa-token-freemarker`）。
//!
//! 技术映射：FreeMarker → **Tera**；Jackson → serde。

pub mod tera;

pub use tera::dialect::{
    DEFAULT_ATTR_NAME, SaTokenTemplateDirectiveModel, SaTokenTemplateModel,
};
