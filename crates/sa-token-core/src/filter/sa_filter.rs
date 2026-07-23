//! SaFilter：过滤器接口（对应 Java `cn.dev33.satoken.filter.SaFilter`）。

use crate::router::sa_router_staff::SaRouterStaff as RouterStaff;

/// 过滤器接口
///
/// 各 Web 框架适配层应实现此接口，并将 SaRouter 逻辑转换为框架原生中间件。
pub trait SaFilter: Send + Sync {
    /// 执行路由匹配链
    fn run(&self, staff: &RouterStaff);
}
