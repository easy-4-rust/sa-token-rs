//! `SaTokenTemplateDirectiveModel` —— 1:1 对应 Java
//! `cn.dev33.satoken.freemarker.dialect.SaTokenTemplateDirectiveModel`
//!
//! Java 实现 `TemplateDirectiveModel.execute`：断言为 true 时渲染标签体。
//! Rust/Tera：封装为可调用断言，由模板 `{% if sa_xxx(...) %}` 控制显示。

use std::sync::Arc;

use tera::{Function, Result as TeraResult, Value};

/// 断言函数类型：入参为标签 `value` 属性，返回是否显示内容。
///
/// 对应 Java `Function<String, Boolean> fun`。
pub type DirectivePredicate = Arc<dyn Fn(Option<&str>) -> bool + Send + Sync>;

/// Sa-Token 模板指令模型（对应 Java `SaTokenTemplateDirectiveModel`）
pub struct SaTokenTemplateDirectiveModel {
    /// 属性名（对应 Java `attrName`，默认 `"value"`）
    pub attr_name: String,
    /// 断言函数（对应 Java `fun`）
    pub fun: DirectivePredicate,
}

impl SaTokenTemplateDirectiveModel {
    /// 构造指令模型（对应 Java 构造器）。
    pub fn new(attr_name: impl Into<String>, fun: DirectivePredicate) -> Self {
        Self {
            attr_name: attr_name.into(),
            fun,
        }
    }

    /// 执行断言（对应 Java `execute` 中 `fun.apply(value)`）。
    pub fn evaluate(&self, value: Option<&str>) -> bool {
        (self.fun)(value)
    }

    /// 转为 Tera 函数：从 kwargs 读取 `attr_name` 指定属性并返回 bool。
    pub fn into_tera_function(self) -> impl Function {
        let attr = self.attr_name.clone();
        let fun = self.fun.clone();
        move |args: &std::collections::HashMap<String, Value>| -> TeraResult<Value> {
            let raw = args.get(&attr).or_else(|| args.get("value"));
            let value = raw.and_then(|v| v.as_str());
            Ok(Value::Bool(fun(value)))
        }
    }
}
