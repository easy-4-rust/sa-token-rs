//! `SaCheckSafeHandler` —— 1:1 对应 Java
//! `cn.dev33.satoken.annotation.handler.SaCheckSafeHandler`

use super::super::sa_check_safe::SaCheckSafeMeta;
use super::sa_annotation_handler_interface::SaAnnotationHandlerInterface;
use crate::exception::{SaResult, SaTokenException};

/// `@SaCheckSafe` 注解处理器（对应 Java `SaCheckSafeHandler`）
pub struct SaCheckSafeHandler {
    /// 注解元数据
    pub meta: SaCheckSafeMeta,
}

impl SaCheckSafeHandler {
    /// 构造处理器。
    pub fn new(meta: SaCheckSafeMeta) -> Self {
        Self { meta }
    }

    /// 在已知是否已通过二级认证时判定（对应 Java `checkSafe`）。
    pub fn check_with_safe(&self, is_safe: bool) -> SaResult<()> {
        if !is_safe {
            let login_type = if self.meta.r#type.is_empty() {
                "login"
            } else {
                self.meta.r#type
            };
            return Err(SaTokenException::not_safe(self.meta.value, login_type));
        }
        Ok(())
    }
}

impl SaAnnotationHandlerInterface for SaCheckSafeHandler {
    const ANNOTATION: &'static str = "SaCheckSafe";

    fn check(&self) -> SaResult<()> {
        Ok(())
    }
}
