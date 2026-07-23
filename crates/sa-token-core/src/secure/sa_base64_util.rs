//! Base64 工具（对应 Java `cn.dev33.satoken.secure.SaBase64Util`）。

use base64::{Engine, engine::general_purpose};

/// Base64 工具
pub struct SaBase64Util;

impl SaBase64Util {
    /// 编码
    pub fn encode(data: &[u8]) -> String {
        general_purpose::STANDARD.encode(data)
    }

    /// 解码
    pub fn decode(s: &str) -> Result<Vec<u8>, base64::DecodeError> {
        general_purpose::STANDARD.decode(s)
    }

    /// URL 安全编码
    pub fn encode_url_safe(data: &[u8]) -> String {
        general_purpose::URL_SAFE_NO_PAD.encode(data)
    }

    /// URL 安全解码
    pub fn decode_url_safe(s: &str) -> Result<Vec<u8>, base64::DecodeError> {
        general_purpose::URL_SAFE_NO_PAD.decode(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip() {
        let s = "hello world";
        let enc = SaBase64Util::encode(s.as_bytes());
        let dec = SaBase64Util::decode(&enc).unwrap();
        assert_eq!(String::from_utf8(dec).unwrap(), s);
    }
}
