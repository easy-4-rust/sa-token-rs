//! `SaFirewallCheckHookForDirectoryTraversal` —— 1:1 对应 Java 同名类
//!
//! 防火墙策略校验钩子：请求 path 目录遍历符检测。

use std::sync::OnceLock;

use crate::context::model::{sa_request::SaRequest, sa_response::SaResponse};
use crate::exception::{RequestPathInvalidException, SaTokenException};

use super::sa_firewall_check_hook::SaFirewallCheckHook;

/// 目录遍历检测 hook（对应 Java `SaFirewallCheckHookForDirectoryTraversal`）。
#[derive(Clone, Copy, Default)]
pub struct SaFirewallCheckHookForDirectoryTraversal;

impl SaFirewallCheckHookForDirectoryTraversal {
    /// 默认单例（对应 Java `instance`）。
    pub fn instance() -> &'static Self {
        static INST: OnceLock<SaFirewallCheckHookForDirectoryTraversal> = OnceLock::new();
        INST.get_or_init(SaFirewallCheckHookForDirectoryTraversal::default)
    }

    /// 检查路径是否有效（对应 Java `isPathValid`）。
    pub fn is_path_valid(path: &str) -> bool {
        if path.is_empty() {
            return false;
        }
        if !path.starts_with('/') {
            return false;
        }
        if path == "/" {
            return true;
        }

        for (index, component) in path.split('/').enumerate() {
            if component.is_empty() {
                if index == 0 {
                    continue;
                }
                return false;
            }
            if component == "." || component == ".." {
                return false;
            }
        }
        true
    }
}

impl SaFirewallCheckHook for SaFirewallCheckHookForDirectoryTraversal {
    fn execute(&self, req: &dyn SaRequest, _res: &dyn SaResponse) -> Result<(), SaTokenException> {
        let request_path = req.get_request_path();
        if !Self::is_path_valid(&request_path) {
            return Err(RequestPathInvalidException::new(format!(
                "非法请求：{request_path}"
            ))
            .into());
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn path_validity_matches_java_cases() {
        assert!(SaFirewallCheckHookForDirectoryTraversal::is_path_valid("/user/info"));
        assert!(!SaFirewallCheckHookForDirectoryTraversal::is_path_valid("/user/info/.."));
        assert!(!SaFirewallCheckHookForDirectoryTraversal::is_path_valid("//user"));
        assert!(SaFirewallCheckHookForDirectoryTraversal::is_path_valid("/"));
    }
}
