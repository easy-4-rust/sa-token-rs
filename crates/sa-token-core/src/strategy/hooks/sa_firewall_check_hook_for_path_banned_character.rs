//! `SaFirewallCheckHookForPathBannedCharacter` —— 1:1 对应 Java 同名类
//!
//! 防火墙策略校验钩子：请求 path 禁止字符校验。

use std::sync::{OnceLock, RwLock};

use crate::context::model::{sa_request::SaRequest, sa_response::SaResponse};
use crate::exception::{RequestPathInvalidException, SaTokenException};
use crate::util::sa_fox_util::SaFoxUtil;

use super::sa_firewall_check_hook::SaFirewallCheckHook;

/// path 禁止字符 hook（对应 Java `SaFirewallCheckHookForPathBannedCharacter`）。
pub struct SaFirewallCheckHookForPathBannedCharacter {
    /// 是否严格禁止 `%`（对应 Java `bannedPercentage`）。
    pub banned_percentage: RwLock<bool>,
}

impl SaFirewallCheckHookForPathBannedCharacter {
    /// 默认单例（对应 Java `instance`）。
    pub fn instance() -> &'static Self {
        static INST: OnceLock<SaFirewallCheckHookForPathBannedCharacter> = OnceLock::new();
        INST.get_or_init(|| Self {
            banned_percentage: RwLock::new(false),
        })
    }

    /// 重载配置（对应 Java `resetConfig`）。
    pub fn reset_config(&self, banned_percentage: bool) {
        *self.banned_percentage.write().unwrap() = banned_percentage;
    }
}

impl SaFirewallCheckHook for SaFirewallCheckHookForPathBannedCharacter {
    fn execute(&self, req: &dyn SaRequest, _res: &dyn SaResponse) -> Result<(), SaTokenException> {
        let request_path = req.get_request_path();
        if SaFoxUtil::has_non_printable_ascii(&request_path) {
            return Err(RequestPathInvalidException::new(format!(
                "请求 path 包含禁止字符：{request_path}"
            ))
            .into());
        }
        if *self.banned_percentage.read().unwrap() && request_path.contains('%') {
            return Err(RequestPathInvalidException::new(format!(
                "请求 path 包含禁止字符 %：{request_path}"
            ))
            .into());
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context::mock::sa_request_for_mock::SaRequestForMock;
    use crate::context::mock::sa_response_for_mock::SaResponseForMock;

    #[test]
    fn rejects_non_printable_ascii() {
        let req = SaRequestForMock::new().with_url("/user/\n/info");
        assert!(
            SaFirewallCheckHookForPathBannedCharacter::instance()
                .execute(&req, &SaResponseForMock::new())
                .is_err()
        );
    }
}
