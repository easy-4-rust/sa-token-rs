//! `SaCheckLogin` —— 1:1 对应 Java `cn.dev33.satoken.annotation.SaCheckLogin`
//!
//! 登录认证校验：只有登录之后才能进入该方法。
//! 编译期挂载由 `sa-token-derive::check_login` 属性宏完成。

/// 注解运行时元数据（对应 Java `@SaCheckLogin`）
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SaCheckLoginMeta {
    /// 多账号体系标识（对应 Java `type()`，默认 `""`）
    pub r#type: &'static str,
}

impl SaCheckLoginMeta {
    /// 创建默认元数据（`type = ""`，对应 Java 注解默认值）。
    pub const fn new() -> Self {
        Self { r#type: "" }
    }

    /// 指定账号体系标识（对应 Java `type()`）。
    pub const fn with_type(r#type: &'static str) -> Self {
        Self { r#type }
    }
}

impl Default for SaCheckLoginMeta {
    fn default() -> Self {
        Self::new()
    }
}
