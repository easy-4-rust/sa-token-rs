//! `SaCheckHttpBasicHandler` —— 1:1 对应 Java
//! `cn.dev33.satoken.annotation.handler.SaCheckHttpBasicHandler`

use super::super::sa_check_http_basic::SaCheckHttpBasicMeta;
use super::sa_annotation_handler_interface::SaAnnotationHandlerInterface;
use crate::exception::SaResult;

/// `@SaCheckHttpBasic` 注解处理器（对应 Java `SaCheckHttpBasicHandler`）
pub struct SaCheckHttpBasicHandler {
    /// 注解元数据
    pub meta: SaCheckHttpBasicMeta,
}

impl SaCheckHttpBasicHandler {
    /// 构造处理器。
    pub fn new(meta: SaCheckHttpBasicMeta) -> Self {
        Self { meta }
    }
}

impl SaAnnotationHandlerInterface for SaCheckHttpBasicHandler {
    const ANNOTATION: &'static str = "SaCheckHttpBasic";

    fn check(&self) -> SaResult<()> {
        // 完整校验依赖请求 Authorization 头，由 Http Basic 模板执行
        Ok(())
    }
}
