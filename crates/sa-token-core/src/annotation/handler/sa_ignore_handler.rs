//! `SaIgnoreHandler` —— 1:1 对应 Java
//! `cn.dev33.satoken.annotation.handler.SaIgnoreHandler`
//!
//! Java：`checkMethod` 为空实现，表示跳过鉴权。

use super::super::sa_ignore::SaIgnoreMeta;
use super::sa_annotation_handler_interface::SaAnnotationHandlerInterface;
use crate::exception::SaResult;

/// `@SaIgnore` 注解处理器（恒返回 Ok）
pub struct SaIgnoreHandler {
    /// 注解元数据
    pub meta: SaIgnoreMeta,
}

impl SaIgnoreHandler {
    /// 构造处理器。
    pub fn new(meta: SaIgnoreMeta) -> Self {
        Self { meta }
    }
}

impl SaAnnotationHandlerInterface for SaIgnoreHandler {
    const ANNOTATION: &'static str = "SaIgnore";

    /// 对应 Java 空 `checkMethod`：不做任何校验。
    fn check(&self) -> SaResult<()> {
        Ok(())
    }
}
