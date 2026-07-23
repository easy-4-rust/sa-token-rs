//! SaFilterAuthStrategy：认证策略函数式接口（对应 Java `cn.dev33.satoken.filter.SaFilterAuthStrategy`）。

/// 认证策略函数（对应 Java `@FunctionalInterface SaFilterAuthStrategy.run`）。
pub type SaFilterAuthStrategy = Box<dyn Fn() + Send + Sync>;
