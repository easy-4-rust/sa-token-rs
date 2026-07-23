//! `SaCheckDisableHandler` —— 1:1 对应 Java
//! `cn.dev33.satoken.annotation.handler.SaCheckDisableHandler`

use super::super::sa_check_disable::SaCheckDisableMeta;
use super::sa_annotation_handler_interface::SaAnnotationHandlerInterface;
use crate::exception::SaResult;

/// `@SaCheckDisable` 注解处理器（对应 Java `SaCheckDisableHandler`）
pub struct SaCheckDisableHandler {
    /// 注解元数据
    pub meta: SaCheckDisableMeta,
}

impl SaCheckDisableHandler {
    /// 构造处理器。
    pub fn new(meta: SaCheckDisableMeta) -> Self {
        Self { meta }
    }

    /// 在已知禁用等级时判定是否拦截（对应 Java `checkDisableLevel` 语义）。
    ///
    /// `current_level >= meta.level` 时视为仍被禁用，应拒绝。
    pub fn check_with_level(&self, login_id: &str, current_level: Option<i32>) -> SaResult<()> {
        if let Some(level) = current_level {
            if level >= self.meta.level {
                let service = self.meta.value.first().copied().unwrap_or("login");
                return Err(crate::exception::SaTokenException::disable_service(
                    login_id, service, -1,
                ));
            }
        }
        Ok(())
    }
}

impl SaAnnotationHandlerInterface for SaCheckDisableHandler {
    const ANNOTATION: &'static str = "SaCheckDisable";

    fn check(&self) -> SaResult<()> {
        Ok(())
    }
}
