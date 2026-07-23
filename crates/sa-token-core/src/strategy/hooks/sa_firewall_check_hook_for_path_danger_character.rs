//! `SaFirewallCheckHookForPathDangerCharacter` —— 1:1 对应 Java 同名类
//!
//! 防火墙策略校验钩子：请求 path 危险字符校验。

use std::sync::{OnceLock, RwLock};

use crate::context::model::{sa_request::SaRequest, sa_response::SaResponse};
use crate::exception::{RequestPathInvalidException, SaTokenException};

use super::sa_firewall_check_hook::SaFirewallCheckHook;

/// path 危险字符 hook（对应 Java `SaFirewallCheckHookForPathDangerCharacter`）。
pub struct SaFirewallCheckHookForPathDangerCharacter {
    /// 不允许出现的危险子串（对应 Java `dangerCharacter`）。
    pub danger_character: RwLock<Vec<String>>,
}

impl SaFirewallCheckHookForPathDangerCharacter {
    /// 默认单例（对应 Java `instance`）。
    pub fn instance() -> &'static Self {
        static INST: OnceLock<SaFirewallCheckHookForPathDangerCharacter> = OnceLock::new();
        INST.get_or_init(|| Self {
            danger_character: RwLock::new(vec![
                "//".into(),
                "\\".into(),
                "%2e".into(),
                "%2E".into(),
                "%2f".into(),
                "%2F".into(),
                "%5c".into(),
                "%5C".into(),
                ";".into(),
                "%3b".into(),
                "%3B".into(),
                "%25".into(),
                "\0".into(),
                "%00".into(),
                "\n".into(),
                "%0a".into(),
                "%0A".into(),
                "\r".into(),
                "%0d".into(),
                "%0D".into(),
                "\u{2028}".into(),
                "\u{2029}".into(),
            ]),
        })
    }

    /// 重载配置（对应 Java `resetConfig`）。
    pub fn reset_config(&self, character: &[&str]) {
        let mut guard = self.danger_character.write().unwrap();
        guard.clear();
        guard.extend(character.iter().map(|s| (*s).to_string()));
    }
}

impl SaFirewallCheckHook for SaFirewallCheckHookForPathDangerCharacter {
    fn execute(&self, req: &dyn SaRequest, _res: &dyn SaResponse) -> Result<(), SaTokenException> {
        let request_path = req.get_request_path();
        let guard = self.danger_character.read().unwrap();
        if guard.iter().any(|item| request_path.contains(item)) {
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
    use crate::context::mock::sa_request_for_mock::SaRequestForMock;
    use crate::context::mock::sa_response_for_mock::SaResponseForMock;

    #[test]
    fn rejects_double_slash() {
        let req = SaRequestForMock::new().with_url("/user//info");
        assert!(
            SaFirewallCheckHookForPathDangerCharacter::instance()
                .execute(&req, &SaResponseForMock::new())
                .is_err()
        );
    }
}
