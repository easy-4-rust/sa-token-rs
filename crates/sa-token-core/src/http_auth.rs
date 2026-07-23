//! HTTP 认证模块（对应 Java `cn.dev33.satoken.httpauth`）。

pub mod basic;
pub mod digest;

pub use basic::{
    sa_http_basic_account::SaHttpBasicAccount, sa_http_basic_template::SaHttpBasicTemplate,
    sa_http_basic_util::SaHttpBasicUtil,
};
pub use digest::{
    sa_http_digest_model::SaHttpDigestModel, sa_http_digest_template::SaHttpDigestTemplate,
    sa_http_digest_util::SaHttpDigestUtil,
};
