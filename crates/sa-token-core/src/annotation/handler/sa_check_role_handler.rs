//! `SaCheckRoleHandler` —— 1:1 对应 Java
//! `cn.dev33.satoken.annotation.handler.SaCheckRoleHandler`

use super::super::sa_check_role::SaCheckRoleMeta;
use super::super::sa_mode::SaMode;
use super::sa_annotation_handler_interface::SaAnnotationHandlerInterface;
use crate::exception::{SaResult, SaTokenException};

/// `@SaCheckRole` 注解处理器（对应 Java `SaCheckRoleHandler`）
pub struct SaCheckRoleHandler {
    /// 注解元数据
    pub meta: SaCheckRoleMeta,
}

impl SaCheckRoleHandler {
    /// 构造处理器。
    pub fn new(meta: SaCheckRoleMeta) -> Self {
        Self { meta }
    }

    /// 在已拿到角色列表时执行校验（对应 Java `_checkMethod` 中角色判定）。
    pub fn check_with_roles(&self, has_roles: &[&str]) -> SaResult<()> {
        let login_type = if self.meta.r#type.is_empty() {
            "login"
        } else {
            self.meta.r#type
        };
        let requires = self.meta.value;
        let ok = match self.meta.mode {
            SaMode::And => requires.iter().all(|r| has_roles.contains(r)),
            SaMode::Or => requires.iter().any(|r| has_roles.contains(r)),
        };
        if !ok {
            return Err(SaTokenException::not_role(requires.join(","), login_type));
        }
        Ok(())
    }
}

impl SaAnnotationHandlerInterface for SaCheckRoleHandler {
    const ANNOTATION: &'static str = "SaCheckRole";

    fn check(&self) -> SaResult<()> {
        Ok(())
    }
}
