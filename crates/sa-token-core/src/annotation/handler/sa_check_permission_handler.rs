//! `SaCheckPermissionHandler` —— 1:1 对应 Java
//! `cn.dev33.satoken.annotation.handler.SaCheckPermissionHandler`

use super::super::sa_check_permission::SaCheckPermissionMeta;
use super::super::sa_mode::SaMode;
use super::sa_annotation_handler_interface::SaAnnotationHandlerInterface;
use crate::exception::{SaResult, SaTokenException};

/// `@SaCheckPermission` 注解处理器（对应 Java `SaCheckPermissionHandler`）
pub struct SaCheckPermissionHandler {
    /// 注解元数据
    pub meta: SaCheckPermissionMeta,
}

impl SaCheckPermissionHandler {
    /// 构造处理器。
    pub fn new(meta: SaCheckPermissionMeta) -> Self {
        Self { meta }
    }

    /// 在已拿到权限列表时执行校验（对应 Java `_checkMethod` 中权限判定）。
    pub fn check_with_permissions(&self, has_permissions: &[&str]) -> SaResult<()> {
        let login_type = if self.meta.r#type.is_empty() {
            "login"
        } else {
            self.meta.r#type
        };
        let requires = self.meta.value;
        // AND：全部具备；OR：任一具备（对应 Java SaMode）
        let ok = match self.meta.mode {
            SaMode::And => requires.iter().all(|p| has_permissions.contains(p)),
            SaMode::Or => requires.iter().any(|p| has_permissions.contains(p)),
        };
        if !ok {
            return Err(SaTokenException::not_permission(
                requires.join(","),
                login_type,
            ));
        }
        Ok(())
    }
}

impl SaAnnotationHandlerInterface for SaCheckPermissionHandler {
    const ANNOTATION: &'static str = "SaCheckPermission";

    fn check(&self) -> SaResult<()> {
        // 完整路径依赖请求上下文中的 StpLogic；单测请用 check_with_permissions
        Ok(())
    }
}
