//! SaStrategy：全局策略（对应 Java `cn.dev33.satoken.strategy.SaStrategy`）。
//!
//! 提供可插拔的策略函数：路由匹配、Token 生成、Session 创建、注解扫描等。

use std::sync::{Arc, OnceLock};

use crate::fun::strategy::{
    sa_create_session_function::SaCreateSessionFunction,
    sa_generate_unique_token_function::SaGenerateUniqueTokenFunction,
    sa_has_element_function::SaHasElementFunction, sa_route_match_function::SaRouteMatchFunction,
};
use crate::router::sa_router::route_match;
use crate::session::sa_session::SaSession;
use crate::util::sa_fox_util::{SaFoxUtil, vague_match};

/// 全局策略
///
/// 对应 Java `SaStrategy.instance`。
pub struct SaStrategy {
    /// 路由匹配函数（对应 Java `routeMatcher`）。
    pub route_match: SaRouteMatchFunction,
    /// 集合包含判断（对应 Java `hasElement`）。
    pub has_element: SaHasElementFunction,
    /// Session 工厂（对应 Java `createSession`）。
    pub create_session: SaCreateSessionFunction,
    /// 生成唯一 Token（对应 Java `generateUniqueToken`）。
    pub generate_unique_token: SaGenerateUniqueTokenFunction,
}

impl Default for SaStrategy {
    fn default() -> Self {
        Self {
            route_match: Box::new(route_match),
            has_element: Box::new(|list, element| {
                if list.is_empty() {
                    return false;
                }
                if list.iter().any(|item| item == element) {
                    return true;
                }
                list.iter().any(|patt| vague_match(patt, element))
            }),
            create_session: Box::new(|id: &str| Arc::new(SaSession::new(id))),
            generate_unique_token: Box::new(|_login_type: &str, _token_value: &str| {
                SaFoxUtil::random_string(64)
            }),
        }
    }
}

static STRATEGY: OnceLock<SaStrategy> = OnceLock::new();

impl SaStrategy {
    /// 获取全局实例（对应 Java `SaStrategy.instance`）。
    pub fn instance() -> &'static SaStrategy {
        STRATEGY.get_or_init(SaStrategy::default)
    }

    /// 设置自定义策略（仅首次生效）。
    pub fn set(strategy: SaStrategy) {
        let _ = STRATEGY.set(strategy);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn has_element_supports_exact_and_vague_match() {
        let strategy = SaStrategy::default();
        let list = vec!["localhost".to_string(), "*.example.com".to_string()];
        assert!((strategy.has_element)(&list, "localhost"));
        assert!((strategy.has_element)(&list, "api.example.com"));
        assert!(!(strategy.has_element)(&list, "evil.test"));
    }
}
