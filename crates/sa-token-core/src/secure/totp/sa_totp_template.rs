//! TOTP 算法模板（对应 Java `cn.dev33.satoken.secure.totp.SaTotpTemplate`）。
//!
//! 基于 RFC 6238 的 TOTP 算法，支持：
//! - 生成随机 Base32 密钥
//! - 生成当前时间的 6 位数字动态口令
//! - 验证用户输入（含时间窗口容错）
//! - 生成 Google Authenticator otpauth URI

use hmac::{Hmac, Mac};
use rand::RngExt;
use sha1::Sha1;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::exception::{SaResult, SaTokenException};
use crate::secure::sa_base32_util::SaBase32Util;

type HmacSha1 = Hmac<Sha1>;

/// TOTP 算法模板
///
/// 对应 Java `SaTotpTemplate`。
#[derive(Debug, Clone)]
pub struct SaTotpTemplate {
    /// 时间窗口步长（秒），默认 30
    pub time_step: u32,
    /// 生成的验证码位数，默认 6
    pub code_digits: u32,
    /// 哈希算法名称，目前仅支持 HmacSHA1（默认）
    pub hmac_algorithm: String,
    /// 密钥长度（字节，推荐 16 或 32）
    pub secret_key_length: usize,
}

impl Default for SaTotpTemplate {
    fn default() -> Self {
        Self::new()
    }
}

impl SaTotpTemplate {
    /// 默认构造函数
    pub fn new() -> Self {
        Self {
            time_step: 30,
            code_digits: 6,
            hmac_algorithm: "HmacSHA1".to_string(),
            secret_key_length: 16,
        }
    }

    /// 自定义参数构造函数
    pub fn with_params(
        time_step: u32,
        code_digits: u32,
        hmac_algorithm: impl Into<String>,
        secret_key_length: usize,
    ) -> Self {
        Self {
            time_step,
            code_digits,
            hmac_algorithm: hmac_algorithm.into(),
            secret_key_length,
        }
    }

    /// 生成随机密钥（Base32 编码，无 padding）
    pub fn generate_secret_key(&self) -> String {
        let mut bytes = vec![0u8; self.secret_key_length];
        rand::rng().fill(bytes.as_mut_slice());
        SaBase32Util::encode(&bytes).replace('=', "")
    }

    /// 生成当前时间的 TOTP 验证码
    pub fn generate_totp(&self, secret_key: &str) -> String {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        self.generate_totp_at(secret_key, now as i64)
    }

    /// 在指定时间戳生成 TOTP 验证码
    pub fn generate_totp_at(&self, secret_key: &str, time: i64) -> String {
        let key_bytes = SaBase32Util::decode(secret_key).unwrap_or_default();
        let counter = (time / self.time_step as i64) as u64;
        let counter_bytes = counter.to_be_bytes();

        let mut mac = HmacSha1::new_from_slice(&key_bytes).expect("HMAC accepts any key length");
        mac.update(&counter_bytes);
        let hash = mac.finalize().into_bytes();

        // 动态截断（RFC 6238）
        let offset = (hash[hash.len() - 1] & 0x0f) as usize;
        let binary = ((hash[offset] & 0x7f) as u32) << 24
            | (hash[offset + 1] as u32) << 16
            | (hash[offset + 2] as u32) << 8
            | (hash[offset + 3] as u32);

        let modulus = 10u32.pow(self.code_digits);
        let otp = binary % modulus;
        format!("{:0>width$}", otp, width = self.code_digits as usize)
    }

    /// 校验用户输入的 TOTP 是否有效
    pub fn validate_totp(&self, secret_key: &str, code: &str, time_window_offset: i32) -> bool {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0) as i64;
        let current_window = now / self.time_step as i64;

        for i in -time_window_offset..=time_window_offset {
            let t = (current_window + i as i64) * self.time_step as i64;
            if self.generate_totp_at(secret_key, t) == code {
                return true;
            }
        }
        false
    }

    /// 校验 TOTP（无效则抛异常）
    pub fn check_totp(
        &self,
        secret_key: &str,
        code: &str,
        time_window_offset: i32,
    ) -> SaResult<()> {
        if !self.validate_totp(secret_key, code, time_window_offset) {
            return Err(SaTokenException::TotpAuth);
        }
        Ok(())
    }

    /// 生成 Google Authenticator 扫码字符串
    pub fn generate_google_secret_key(&self, account: &str) -> String {
        self.generate_google_secret_key_with_secret(account, &self.generate_secret_key())
    }

    /// 使用指定密钥生成扫码字符串
    pub fn generate_google_secret_key_with_secret(
        &self,
        account: &str,
        secret_key: &str,
    ) -> String {
        format!("otpauth://totp/{account}?secret={secret_key}")
    }

    /// 生成含 issuer 的扫码字符串
    pub fn generate_google_secret_key_with_issuer(
        &self,
        account: &str,
        issuer: &str,
        secret_key: &str,
    ) -> String {
        format!("otpauth://totp/{issuer}:{account}?secret={secret_key}&issuer={issuer}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generate_secret_key_length() {
        let tpl = SaTotpTemplate::new();
        let key = tpl.generate_secret_key();
        // Base32 编码后：16 字节 -> 26 字符
        assert!(!key.is_empty());
        assert!(key.len() >= 24);
    }

    #[test]
    fn generate_and_validate() {
        let tpl = SaTotpTemplate::new();
        let key = tpl.generate_secret_key();
        let code = tpl.generate_totp(&key);
        assert_eq!(code.len(), 6);
        assert!(tpl.validate_totp(&key, &code, 1));
    }

    #[test]
    fn wrong_code_fails() {
        let tpl = SaTotpTemplate::new();
        let key = tpl.generate_secret_key();
        assert!(!tpl.validate_totp(&key, "000000", 0));
    }

    #[test]
    fn google_auth_uri() {
        let tpl = SaTotpTemplate::new();
        let uri = tpl.generate_google_secret_key_with_secret("user@example.com", "ABCD");
        assert!(uri.starts_with("otpauth://totp/"));
        assert!(uri.contains("secret=ABCD"));

        let uri2 = tpl.generate_google_secret_key_with_issuer("alice", "MyApp", "EFGH");
        assert!(uri2.contains("issuer=MyApp"));
        assert!(uri2.contains("secret=EFGH"));
    }
}
