//! `SaCheckDisable` —— 1:1 对应 Java `cn.dev33.satoken.annotation.SaCheckDisable`
//!
//! 服务禁用校验：判断当前账号是否被禁用了指定服务；被禁用则抛异常。

use crate::util::sa_token_consts::{DEFAULT_DISABLE_LEVEL, DEFAULT_DISABLE_SERVICE};

/// 注解运行时元数据（对应 Java `@SaCheckDisable`）
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SaCheckDisableMeta {
    /// 多账号体系标识（对应 Java `type()`，默认 `""`）
    pub r#type: &'static str,
    /// 要校验的服务标识列表（对应 Java `value()`）
    pub value: &'static [&'static str],
    /// 封禁等级阈值（对应 Java `level()`）
    pub level: i32,
}

impl SaCheckDisableMeta {
    /// 使用 Java 默认值创建元数据（`type=""`, `value=["login"]`, `level=1`）。
    ///
    /// 对应 Java 注解属性默认值。
    pub const fn new() -> Self {
        Self {
            r#type: "",
            value: &[DEFAULT_DISABLE_SERVICE],
            level: DEFAULT_DISABLE_LEVEL,
        }
    }

    /// 指定账号体系标识（对应 Java `type()`）。
    pub const fn with_type(mut self, r#type: &'static str) -> Self {
        self.r#type = r#type;
        self
    }

    /// 指定服务标识列表（对应 Java `value()`）。
    pub const fn with_value(mut self, value: &'static [&'static str]) -> Self {
        self.value = value;
        self
    }

    /// 指定封禁等级（对应 Java `level()`）。
    pub const fn with_level(mut self, level: i32) -> Self {
        self.level = level;
        self
    }
}

impl Default for SaCheckDisableMeta {
    fn default() -> Self {
        Self::new()
    }
}
