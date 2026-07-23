//! `SaCheckSafe` —— 1:1 对应 Java `cn.dev33.satoken.annotation.SaCheckSafe`
//!
//! 二级认证校验：客户端必须完成二级认证后才能进入方法。

use crate::util::sa_token_consts::DEFAULT_SAFE_AUTH_SERVICE;

/// 注解运行时元数据（对应 Java `@SaCheckSafe`）
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SaCheckSafeMeta {
    /// 多账号体系标识（对应 Java `type()`，默认 `""`）
    pub r#type: &'static str,
    /// 要校验的二级认证服务（对应 Java `value()`，默认 `"important"`）
    pub value: &'static str,
}

impl SaCheckSafeMeta {
    /// 使用 Java 默认值创建元数据。
    pub const fn new() -> Self {
        Self {
            r#type: "",
            value: DEFAULT_SAFE_AUTH_SERVICE,
        }
    }

    /// 指定账号体系标识（对应 Java `type()`）。
    pub const fn with_type(mut self, r#type: &'static str) -> Self {
        self.r#type = r#type;
        self
    }

    /// 指定二级认证服务名（对应 Java `value()`）。
    pub const fn with_value(mut self, value: &'static str) -> Self {
        self.value = value;
        self
    }
}

impl Default for SaCheckSafeMeta {
    fn default() -> Self {
        Self::new()
    }
}
