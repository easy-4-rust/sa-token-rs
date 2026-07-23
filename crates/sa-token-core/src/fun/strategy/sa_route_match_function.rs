//! RouteMatch 函数（对应 Java `cn.dev33.satoken.fun.strategy.SaRouteMatchFunction`）。

/// 路由匹配函数
pub type SaRouteMatchFunction = Box<dyn Fn(&str, &str) -> bool + Send + Sync>;
