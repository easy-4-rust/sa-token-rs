//! 方言模型（对应 Java `cn.dev33.satoken.freemarker.dialect`）。

pub mod sa_token_template_directive_model;
pub mod sa_token_template_model;

pub use sa_token_template_directive_model::SaTokenTemplateDirectiveModel;
pub use sa_token_template_model::{DEFAULT_ATTR_NAME, SaTokenTemplateModel};
