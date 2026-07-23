//! `SaIgnore` —— 1:1 对应 Java `cn.dev33.satoken.annotation.SaIgnore`
//!
//! 标识忽略鉴权：标注后跳过 Sa-Token 相关校验。

/// 注解运行时元数据（对应 Java `@SaIgnore`，无属性）
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct SaIgnoreMeta;

impl SaIgnoreMeta {
    /// 创建忽略鉴权元数据。
    pub const fn new() -> Self {
        Self
    }
}
