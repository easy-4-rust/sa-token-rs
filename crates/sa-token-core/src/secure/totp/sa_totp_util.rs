//! TOTP 静态门面（对应 Java `cn.dev33.satoken.secure.totp.SaTotpUtil`）。

use super::sa_totp_template::SaTotpTemplate;
use crate::exception::SaResult;

/// TOTP 静态门面
pub struct SaTotpUtil;

impl SaTotpUtil {
    /// 生成随机密钥
    pub fn generate_secret_key() -> String {
        SaTotpTemplate::new().generate_secret_key()
    }

    /// 生成当前 TOTP 验证码
    pub fn generate_totp(secret_key: &str) -> String {
        SaTotpTemplate::new().generate_totp(secret_key)
    }

    /// 校验 TOTP
    pub fn validate_totp(secret_key: &str, code: &str, time_window_offset: i32) -> bool {
        SaTotpTemplate::new().validate_totp(secret_key, code, time_window_offset)
    }

    /// 校验 TOTP（无效则抛异常）
    pub fn check_totp(secret_key: &str, code: &str, time_window_offset: i32) -> SaResult<()> {
        SaTotpTemplate::new().check_totp(secret_key, code, time_window_offset)
    }

    /// 生成 Google Authenticator URI
    pub fn generate_google_secret_key(account: &str) -> String {
        SaTotpTemplate::new().generate_google_secret_key(account)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn util_facade_works() {
        let key = SaTotpUtil::generate_secret_key();
        let code = SaTotpUtil::generate_totp(&key);
        assert!(SaTotpUtil::validate_totp(&key, &code, 1));
    }
}
