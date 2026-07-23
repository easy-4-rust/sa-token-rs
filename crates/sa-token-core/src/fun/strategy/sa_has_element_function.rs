//! HasElement 函数（对应 Java `cn.dev33.satoken.fun.strategy.SaHasElementFunction`）。
//!
//! 判断集合中是否包含指定元素（支持模糊匹配）。

/// 集合包含判断函数（对应 Java `(list, element) -> boolean`）。
pub type SaHasElementFunction = Box<dyn Fn(&[String], &str) -> bool + Send + Sync>;
