//! `SaAnnotationHandlerInterface` —— 1:1 对应 Java
//! `cn.dev33.satoken.annotation.handler.SaAnnotationHandlerInterface`
//!
//! Java 泛型 `SaAnnotationHandlerInterface<H extends Annotation>`：
//! - `getHandlerAnnotationClass()` → Rust `ANNOTATION` 常量
//! - `checkMethod(at, element)` → Rust `check()`

use crate::exception::SaResult;

/// 注解处理器统一接口（对应 Java `SaAnnotationHandlerInterface`）
pub trait SaAnnotationHandlerInterface {
    /// 该处理器针对的注解类型名（对应 Java `getHandlerAnnotationClass()`）
    const ANNOTATION: &'static str;

    /// 校验入口（对应 Java `checkMethod`）
    fn check(&self) -> SaResult<()>;
}
