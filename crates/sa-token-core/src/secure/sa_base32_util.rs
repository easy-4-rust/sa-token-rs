//! Base32 工具（对应 Java `cn.dev33.satoken.secure.SaBase32Util`）。
//!
//! TOTP 标准（RFC 6238）要求 Secret 使用 Base32 编码。

/// Base32 工具
pub struct SaBase32Util;

const ALPHABET: &[u8; 32] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ234567";

impl SaBase32Util {
    /// 编码字节数组为 Base32 字符串
    pub fn encode(data: &[u8]) -> String {
        if data.is_empty() {
            return String::new();
        }
        let mut output = String::with_capacity((data.len() * 8).div_ceil(5));
        let mut buffer: u64 = 0;
        let mut bits_in_buffer: u32 = 0;

        for &byte in data {
            buffer = (buffer << 8) | u64::from(byte);
            bits_in_buffer += 8;
            while bits_in_buffer >= 5 {
                bits_in_buffer -= 5;
                let idx = ((buffer >> bits_in_buffer) & 0x1f) as usize;
                output.push(ALPHABET[idx] as char);
            }
        }
        if bits_in_buffer > 0 {
            let idx = ((buffer << (5 - bits_in_buffer)) & 0x1f) as usize;
            output.push(ALPHABET[idx] as char);
        }
        output
    }

    /// 解码 Base32 字符串
    pub fn decode(s: &str) -> Result<Vec<u8>, String> {
        let cleaned: String = s
            .chars()
            .filter(|c| !c.is_whitespace() && *c != '=')
            .collect();
        let cleaned = cleaned.to_uppercase();
        if cleaned.is_empty() {
            return Ok(Vec::new());
        }

        let mut output = Vec::with_capacity(cleaned.len() * 5 / 8);
        let mut buffer: u64 = 0;
        let mut bits_in_buffer: u32 = 0;

        for c in cleaned.chars() {
            let idx = ALPHABET.iter().position(|&a| a as u32 == c as u32);
            let idx = match idx {
                Some(i) => i as u64,
                None => return Err(format!("无效 Base32 字符: {c}")),
            };
            buffer = (buffer << 5) | idx;
            bits_in_buffer += 5;
            if bits_in_buffer >= 8 {
                bits_in_buffer -= 8;
                output.push(((buffer >> bits_in_buffer) & 0xff) as u8);
            }
        }
        Ok(output)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode_known() {
        // "foobar" 的 Base32 (RFC 4648)
        assert_eq!(SaBase32Util::encode(b"foobar"), "MZXW6YTBOI");
    }

    #[test]
    fn roundtrip() {
        let original = b"hello world 123";
        let enc = SaBase32Util::encode(original);
        let dec = SaBase32Util::decode(&enc).unwrap();
        assert_eq!(dec, original);
    }

    #[test]
    fn decode_known() {
        let dec = SaBase32Util::decode("MZXW6YTBOI").unwrap();
        assert_eq!(dec, b"foobar");
    }
}
