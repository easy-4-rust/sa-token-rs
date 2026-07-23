//! SaFilterErrorStrategy：异常处理策略函数式接口（对应 Java `cn.dev33.satoken.filter.SaFilterErrorStrategy`）。
//!
//! 返回值会在 `to_string()` 后返回给前端。

use crate::exception::SaTokenException;

/// 异常处理策略（对应 Java `@FunctionalInterface SaFilterErrorStrategy.run`）。
pub type SaFilterErrorStrategy =
    Box<dyn Fn(&SaTokenException) -> String + Send + Sync>;
