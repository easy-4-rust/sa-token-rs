//! 路由匹配工具类（对应 Java `cn.dev33.satoken.router.SaRouter`）。
//!
//! 提供静态方法直接进行路径匹配、HTTP 方法匹配，以及启动一个
//! [`SaRouterStaff`] 流式匹配链。

use super::sa_http_method::SaHttpMethod;
use super::sa_router_staff::SaRouterStaff;
use crate::context::sa_holder::SaHolder;
use crate::exception::{SaResult, SaTokenException};

/// 路由匹配工具
///
/// 对应 Java `SaRouter`（cn.dev33.satoken.router.SaRouter）。
pub struct SaRouter;

impl SaRouter {
    /// 单个 pattern 匹配
    pub fn is_match(pattern: &str, path: &str) -> bool {
        route_match(pattern, path)
    }

    /// 多 pattern 匹配（任一命中即返回 true）
    pub fn is_match_any(patterns: &[&str], path: &str) -> bool {
        patterns.iter().any(|p| route_match(p, path))
    }

    /// HTTP 方法匹配
    pub fn is_match_method(methods: &[SaHttpMethod], method_str: &str) -> bool {
        methods
            .iter()
            .any(|m| m.is_all() || m.to_string().eq_ignore_ascii_case(method_str))
    }

    /// 使用当前请求 URI 进行单个 pattern 匹配
    pub fn is_match_curr(pattern: &str) -> bool {
        let req = SaHolder::get_request();
        Self::is_match(pattern, req.get_request_path().as_str())
    }

    /// 使用当前请求 URI 进行多 pattern 匹配
    pub fn is_match_curr_any(patterns: &[&str]) -> bool {
        let req = SaHolder::get_request();
        Self::is_match_any(patterns, req.get_request_path().as_str())
    }

    /// 使用当前请求方法匹配
    pub fn is_match_curr_method(methods: &[SaHttpMethod]) -> bool {
        let req = SaHolder::get_request();
        Self::is_match_method(methods, req.get_method().as_str())
    }

    // -------- 流式入口 --------

    /// 创建新匹配链
    pub fn new_match() -> SaRouterStaff {
        SaRouterStaff::new()
    }

    /// 直接以 path 模式开始匹配
    pub fn match_paths(patterns: &[&str]) -> SaRouterStaff {
        SaRouterStaff::new().match_paths(patterns)
    }

    /// 直接以 path 模式排除
    pub fn not_match(patterns: &[&str]) -> SaRouterStaff {
        SaRouterStaff::new().not_match(patterns)
    }

    /// 直接以方法模式开始匹配
    pub fn match_methods(methods: &[SaHttpMethod]) -> SaRouterStaff {
        SaRouterStaff::new().match_methods(methods)
    }

    /// 直接以方法模式排除
    pub fn not_match_methods(methods: &[SaHttpMethod]) -> SaRouterStaff {
        SaRouterStaff::new().not_match_methods(methods)
    }

    /// 无条件停止当前路由匹配流程。
    ///
    /// # Errors
    ///
    /// 始终返回 [`SaTokenException::StopMatch`]，由最外层 adapter 终止认证链。
    pub fn stop() -> SaResult<()> {
        Err(SaTokenException::StopMatch)
    }

    /// 无条件结束当前路由流程，并携带空响应。
    ///
    /// # Errors
    ///
    /// 始终返回 [`SaTokenException::BackResult`]。
    pub fn back() -> SaResult<()> {
        Self::back_with("")
    }

    /// 无条件结束当前路由流程，并携带响应结果。
    ///
    /// # Errors
    ///
    /// 始终返回 [`SaTokenException::BackResult`]。
    pub fn back_with(result: impl Into<String>) -> SaResult<()> {
        Err(SaTokenException::BackResult {
            result: result.into(),
        })
    }
}

/// Ant-style 路径匹配
///
/// 支持 `*`（匹配单段）和 `**`（匹配多段）。
pub fn route_match(pattern: &str, path: &str) -> bool {
    if pattern == path {
        return true;
    }

    // 全通配
    if pattern == "/**" || pattern == "**" {
        return true;
    }

    let pat_segments: Vec<&str> = pattern.trim_matches('/').split('/').collect();
    let path_segments: Vec<&str> = path.trim_matches('/').split('/').collect();

    match_segments(&pat_segments, &path_segments)
}

fn match_segments(pat: &[&str], path: &[&str]) -> bool {
    let mut i = 0;
    let mut j = 0;
    let mut star_idx: Option<usize> = None;
    let mut match_idx = 0;

    while j < path.len() {
        if i < pat.len() {
            match pat[i] {
                "**" => {
                    star_idx = Some(i);
                    match_idx = j;
                    i += 1;
                    continue;
                }
                "*" => {
                    i += 1;
                    j += 1;
                    continue;
                }
                p if p == path[j] => {
                    i += 1;
                    j += 1;
                    continue;
                }
                _ => {}
            }
        }
        if let Some(s) = star_idx {
            i = s + 1;
            match_idx += 1;
            j = match_idx;
        } else {
            return false;
        }
    }

    // 处理尾部 **
    while i < pat.len() && pat[i] == "**" {
        i += 1;
    }

    i == pat.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exact_match() {
        assert!(route_match("/user/list", "/user/list"));
        assert!(!route_match("/user/list", "/user/edit"));
    }

    #[test]
    fn single_star() {
        assert!(route_match("/user/*", "/user/list"));
        assert!(!route_match("/user/*", "/user/list/1"));
        assert!(!route_match("/user/*", "/admin/list"));
    }

    #[test]
    fn double_star() {
        assert!(route_match("/user/**", "/user/list"));
        assert!(route_match("/user/**", "/user/list/1"));
        assert!(route_match("/user/**", "/user/list/1/2"));
        assert!(!route_match("/user/**", "/admin/list"));
    }

    #[test]
    fn all_wildcard() {
        assert!(route_match("/**", "/any/path"));
        assert!(route_match("**", "/any/path"));
    }

    #[test]
    fn method_matching_supports_connect_and_all() {
        assert!(SaRouter::is_match_method(
            &[SaHttpMethod::Connect],
            "connect"
        ));
        assert!(SaRouter::is_match_method(&[SaHttpMethod::All], "CUSTOM"));
        assert!(!SaRouter::is_match_method(&[SaHttpMethod::Get], "POST"));
    }

    #[test]
    fn global_stop_and_back_always_exit() {
        assert_eq!(
            SaRouter::stop().expect_err("stop must exit"),
            SaTokenException::StopMatch
        );
        assert_eq!(
            SaRouter::back_with("denied").expect_err("back must exit"),
            SaTokenException::BackResult {
                result: "denied".to_owned()
            }
        );
    }
}
