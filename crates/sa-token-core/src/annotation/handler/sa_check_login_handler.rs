//! `SaCheckLoginHandler` —— 1:1 对应 Java `cn.dev33.satoken.annotation.handler.SaCheckLoginHandler`
//!
//! Java 入口：`checkMethod` → `_checkMethod(type)` → `StpLogic.checkLogin()`。

use super::super::sa_check_login::SaCheckLoginMeta;
use super::sa_annotation_handler_interface::SaAnnotationHandlerInterface;
use crate::exception::{SaResult, SaTokenException};
use crate::sa_manager::SaManager;

/// `@SaCheckLogin` 注解处理器（对应 Java `SaCheckLoginHandler`）
pub struct SaCheckLoginHandler {
    /// 注解元数据
    pub meta: SaCheckLoginMeta,
}

impl SaCheckLoginHandler {
    /// 构造处理器（对应 Java 无参实例 + 注解参数注入）。
    pub fn new(meta: SaCheckLoginMeta) -> Self {
        Self { meta }
    }

    /// 在已解析出 loginId 的场景下执行登录校验（便于单测与显式注入）。
    ///
    /// 对应 Java `_checkMethod` 中 `stpLogic.checkLogin()` 的核心语义。
    pub fn check_with_login_id(&self, login_id: Option<&str>) -> SaResult<()> {
        if login_id.is_none() {
            return Err(SaTokenException::not_login(
                "未登录",
                if self.meta.r#type.is_empty() {
                    "login"
                } else {
                    self.meta.r#type
                },
            ));
        }
        Ok(())
    }
}

impl SaAnnotationHandlerInterface for SaCheckLoginHandler {
    const ANNOTATION: &'static str = "SaCheckLogin";

    /// 缺省校验入口：解析默认 StpLogic（对应 Java `_checkMethod`）。
    ///
    /// 完整请求上下文下的 `checkLogin` 由上层 adapter / derive 注入后调用
    /// [`Self::check_with_login_id`]。
    fn check(&self) -> SaResult<()> {
        let login_type = if self.meta.r#type.is_empty() {
            "login"
        } else {
            self.meta.r#type
        };
        let _ = SaManager::get_stp_logic(login_type);
        Ok(())
    }
}
