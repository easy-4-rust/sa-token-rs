//! `SaTokenTagProcessor` —— 1:1 对应 Java
//! `cn.dev33.satoken.thymeleaf.dialect.SaTokenTagProcessor`
//!
//! Java：属性处理器在断言为 false 时 `removeElement`。
//! Rust/Askama：封装谓词，由调用方在编译期模板中用 `{% if %}` 控制显隐。

use std::sync::Arc;

/// 标签断言函数类型：入参为属性值，返回是否保留/显示元素。
///
/// 对应 Java `Function<String, Boolean> fun`。
pub type TagPredicate = Arc<dyn Fn(Option<&str>) -> bool + Send + Sync>;

/// Sa-Token 标签处理器（对应 Java `SaTokenTagProcessor`）。
pub struct SaTokenTagProcessor {
    /// 方言前缀（对应 Java `dialectPrefix`，默认 `"sa"`）
    pub dialect_prefix: String,
    /// 属性名（对应 Java `attrName`，如 `login` / `hasRole`）
    pub attr_name: String,
    /// 断言函数（对应 Java `fun`）
    pub fun: TagPredicate,
}

impl SaTokenTagProcessor {
    /// 构造处理器（对应 Java 构造器）。
    pub fn new(
        dialect_prefix: impl Into<String>,
        attr_name: impl Into<String>,
        fun: TagPredicate,
    ) -> Self {
        Self {
            dialect_prefix: dialect_prefix.into(),
            attr_name: attr_name.into(),
            fun,
        }
    }

    /// 执行断言（对应 Java `doProcess` 中 `fun.apply(attributeValue)`）。
    ///
    /// 返回 `true` 表示保留元素；`false` 表示应删除/不渲染（对应 Java `removeElement`）。
    pub fn evaluate(&self, attribute_value: Option<&str>) -> bool {
        (self.fun)(attribute_value)
    }

    /// 是否应渲染标签体（语义同 [`Self::evaluate`]）。
    pub fn should_render(&self, attribute_value: Option<&str>) -> bool {
        self.evaluate(attribute_value)
    }
}
