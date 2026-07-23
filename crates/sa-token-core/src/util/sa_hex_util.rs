//! `SaHexUtil` —— 1:1 对应 Java `cn.dev33.satoken.util.SaHexUtil`
//!
//! Java 的 `SaHexUtil` 是工具类（`public static` 方法集合）。Rust 中以模块命名空间暴露，
//! 保持与 Java 相同的调用形式：`SaHexUtil::encode(bytes)` / `SaHexUtil::decode(s)`。

/// 字节 → hex 字符串（与 Java `SaHexUtil` 一致，使用大写）
pub fn encode(bytes: &[u8]) -> String {
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        s.push_str(&format!("{b:02X}"));
    }
    s
}

/// hex 字符串 → 字节（接受大小写）
pub fn decode(s: &str) -> Result<Vec<u8>, String> {
    let bytes = s.as_bytes();
    if bytes.len() % 2 != 0 {
        return Err("hex 字符串长度必须是偶数".into());
    }
    let mut out = Vec::with_capacity(bytes.len() / 2);
    let mut i = 0;
    while i < bytes.len() {
        let hi = hex_digit(bytes[i])?;
        let lo = hex_digit(bytes[i + 1])?;
        out.push((hi << 4) | lo);
        i += 2;
    }
    Ok(out)
}

fn hex_digit(c: u8) -> Result<u8, String> {
    match c {
        b'0'..=b'9' => Ok(c - b'0'),
        b'a'..=b'f' => Ok(c - b'a' + 10),
        b'A'..=b'F' => Ok(c - b'A' + 10),
        _ => Err(format!("非法的 hex 字符: {}", c as char)),
    }
}

/// 占位包装类型（与 Java `SaHexUtil` 静态方法集合语义保持一致）。
///
/// Java 端 `SaHexUtil.encode(bytes)` 在 Rust 中可直接通过模块函数 `SaHexUtil::encode(bytes)` 调用；
/// 保留此空 struct 以便外部 `use crate::util::sa_hex_util::SaHexUtil;` 路径仍然合法。
pub struct SaHexUtil;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode_decode_roundtrip() {
        let s = encode(b"hello");
        assert_eq!(s, "68656C6C6F");
        assert_eq!(decode(&s).unwrap(), b"hello");
    }

    #[test]
    fn uppercase_works() {
        assert_eq!(decode("FF").unwrap(), vec![0xff]);
    }

    #[test]
    fn invalid_chars() {
        assert!(decode("xyz").is_err());
        assert!(decode("xx").is_err());
    }
}
