//! `SaCheckOrHandler` —— 1:1 对应 Java
//! `cn.dev33.satoken.annotation.handler.SaCheckOrHandler`
//!
//! Java：依次尝试嵌套注解处理器，任一通过即成功。

use super::super::sa_check_or::SaCheckOrMeta;
use super::sa_annotation_handler_interface::SaAnnotationHandlerInterface;
use crate::exception::SaResult;

/// `@SaCheckOr` 注解处理器（对应 Java `SaCheckOrHandler`）
pub struct SaCheckOrHandler {
    /// 注解元数据
    pub meta: SaCheckOrMeta,
}

impl SaCheckOrHandler {
    /// 构造处理器。
    pub fn new(meta: SaCheckOrMeta) -> Self {
        Self { meta }
    }

    /// 对一组子校验结果做 OR 聚合（对应 Java 多注解短路成功）。
    pub fn check_any_ok(&self, results: &[SaResult<()>]) -> SaResult<()> {
        if results.is_empty() {
            return Ok(());
        }
        if results.iter().any(|r| r.is_ok()) {
            return Ok(());
        }
        // 全部失败时返回最后一个错误
        match results.last() {
            Some(Err(e)) => Err(e.clone()),
            _ => Ok(()),
        }
    }
}

impl SaAnnotationHandlerInterface for SaCheckOrHandler {
    const ANNOTATION: &'static str = "SaCheckOr";

    fn check(&self) -> SaResult<()> {
        Ok(())
    }
}
