//! `SaCheckPermission` —— 1:1 对应 Java `cn.dev33.satoken.annotation.SaCheckPermission`
//!
//! 权限码校验：具备指定权限后才能进入方法。

use super::sa_mode::SaMode;

/// 注解运行时元数据（对应 Java `@SaCheckPermission`）
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SaCheckPermissionMeta {
    /// 需要校验的权限码（对应 Java `value()`）
    pub value: &'static [&'static str],
    /// 多账号体系标识（对应 Java `type()`，默认 `""`）
    pub r#type: &'static str,
    /// AND / OR 模式（对应 Java `mode()`，默认 `AND`）
    pub mode: SaMode,
}

impl SaCheckPermissionMeta {
    /// 以权限码列表创建元数据，`type=""`、`mode=And`。
    pub const fn new(value: &'static [&'static str]) -> Self {
        Self {
            value,
            r#type: "",
            mode: SaMode::And,
        }
    }

    /// 指定账号体系（对应 Java `type()`）。
    pub const fn with_type(mut self, r#type: &'static str) -> Self {
        self.r#type = r#type;
        self
    }

    /// 指定校验模式（对应 Java `mode()`）。
    pub const fn with_mode(mut self, mode: SaMode) -> Self {
        self.mode = mode;
        self
    }
}
