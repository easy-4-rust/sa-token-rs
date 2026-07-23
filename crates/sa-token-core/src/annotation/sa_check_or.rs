//! `SaCheckOr` —— 1:1 对应 Java `cn.dev33.satoken.annotation.SaCheckOr`
//!
//! 批量注解鉴权：只要满足其中一个嵌套注解即可通过。
//! Rust 侧以运行时 handler 组合模拟 Java 的嵌套注解数组。

/// 注解运行时元数据（对应 Java `@SaCheckOr` 的组合入口）。
///
/// Java 通过 `login()/role()/permission()` 等嵌套数组声明；
/// Rust 由 derive/handler 在编译期或运行时组装子校验。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct SaCheckOrMeta {
    /// 可选账号体系提示（Rust 扩展；Java 无此字段，子注解各自带 `type`）
    pub r#type: &'static str,
}

impl SaCheckOrMeta {
    /// 创建空组合元数据。
    pub const fn new() -> Self {
        Self { r#type: "" }
    }

    /// 指定账号体系提示。
    pub const fn with_type(r#type: &'static str) -> Self {
        Self { r#type }
    }
}
