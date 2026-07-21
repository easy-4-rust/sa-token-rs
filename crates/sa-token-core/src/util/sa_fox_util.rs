//! 工具类（对应 Java `cn.dev33.satoken.util.SaFoxUtil`）。
use rand::Rng;

/// 生成指定长度的随机字符串（字母+数字）
pub fn random_string(len: usize) -> String {
    let chars = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
    let mut rng = rand::thread_rng();
    (0..len)
        .map(|_| chars[rng.gen_range(0..chars.len())] as char)
        .collect()
}

/// 判断字符串是否为空
pub fn is_empty(s: &str) -> bool {
    s.is_empty()
}

/// 判断字符串是否不为空
pub fn is_not_empty(s: &str) -> bool {
    !s.is_empty()
}

/// 判断两个字符串是否相等
pub fn equals(a: &str, b: &str) -> bool {
    a == b
}

/// 获取当前时间戳（秒）
pub fn now_timestamp() -> i64 {
    chrono::Utc::now().timestamp()
}

/// 获取当前时间戳（毫秒）
pub fn now_timestamp_millis() -> i64 {
    chrono::Utc::now().timestamp_millis()
}

/// 拼接参数为 URL 查询字符串
pub fn join_param(params: &[(&str, &str)]) -> String {
    params
        .iter()
        .map(|(k, v)| format!("{}={}", urlencoding_encode(k), urlencoding_encode(v)))
        .collect::<Vec<_>>()
        .join("&")
}

/// 简单的 URL 编码
fn urlencoding_encode(s: &str) -> String {
    let mut result = String::new();
    for byte in s.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                result.push(byte as char);
            }
            _ => {
                result.push_str(&format!("%{:02X}", byte));
            }
        }
    }
    result
}

/// 拼接两个 URL
pub fn splice_two_url(url1: &str, url2: &str) -> String {
    if url1.is_empty() {
        return url2.to_string();
    }
    if url2.is_empty() {
        return url1.to_string();
    }
    let url1 = url1.trim_end_matches('/');
    let url2 = url2.trim_start_matches('/');
    format!("{}/{}", url1, url2)
}

/// 判断是否为基本类型
pub fn is_basic_type(type_name: &str) -> bool {
    matches!(
        type_name,
        "i8" | "i16"
            | "i32"
            | "i64"
            | "i128"
            | "u8"
            | "u16"
            | "u32"
            | "u64"
            | "u128"
            | "f32"
            | "f64"
            | "bool"
            | "char"
            | "String"
            | "&str"
    )
}
