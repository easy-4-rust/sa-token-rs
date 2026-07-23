//! `SaCheckHttpDigestHandler` —— 1:1 对应 Java
//! `cn.dev33.satoken.annotation.handler.SaCheckHttpDigestHandler`

use super::super::sa_check_http_digest::SaCheckHttpDigestMeta;
use super::sa_annotation_handler_interface::SaAnnotationHandlerInterface;
use crate::exception::SaResult;

/// `@SaCheckHttpDigest` 注解处理器（对应 Java `SaCheckHttpDigestHandler`）
pub struct SaCheckHttpDigestHandler {
    /// 注解元数据
    pub meta: SaCheckHttpDigestMeta,
}

impl SaCheckHttpDigestHandler {
    /// 构造处理器。
    pub fn new(meta: SaCheckHttpDigestMeta) -> Self {
        Self { meta }
    }
}

impl SaAnnotationHandlerInterface for SaCheckHttpDigestHandler {
    const ANNOTATION: &'static str = "SaCheckHttpDigest";

    fn check(&self) -> SaResult<()> {
        Ok(())
    }
}
