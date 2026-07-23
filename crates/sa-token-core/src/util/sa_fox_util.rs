//! 工具类（对应 Java `cn.dev33.satoken.util.SaFoxUtil`）。

use chrono::{Local, TimeZone, Utc};
use rand::RngExt;
use regex::Regex;
use std::sync::LazyLock;

/// URL 校验正则（对应 Java `SaFoxUtil.URL_REGEX`）
pub static URL_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)(https?|ftp|file)://[-A-Za-z0-9+&@#/%?=~_|!:,.;]+[-A-Za-z0-9+&@#/%=~_|]")
        .expect("URL_REGEX 编译失败")
});

/// 日志等级列表（对应 Java `SaFoxUtil.logLevelList`）
pub const LOG_LEVEL_LIST: [&str; 7] = ["", "trace", "debug", "info", "warn", "error", "fatal"];

/// Sa-Token 内部工具类（对应 Java 静态方法集合）。
pub struct SaFoxUtil;

impl SaFoxUtil {
    /// 生成指定长度的随机字符串（字母+数字）
    pub fn random_string(len: usize) -> String {
        random_string(len)
    }

    /// 生成指定区间的随机整数（含边界）
    pub fn random_number(min: i32, max: i32) -> i32 {
        random_number(min, max)
    }

    /// 判断字符串是否为空
    pub fn is_empty(s: &str) -> bool {
        is_empty(s)
    }

    /// 判断字符串是否不为空
    pub fn is_not_empty(s: &str) -> bool {
        is_not_empty(s)
    }

    /// 是否包含不可打印 ASCII 字符（对应 Java `SaFoxUtil.hasNonPrintableASCII`）。
    pub fn has_non_printable_ascii(s: &str) -> bool {
        has_non_printable_ascii(s)
    }

    /// 判断两个字符串是否相等
    pub fn equals(a: &str, b: &str) -> bool {
        equals(a, b)
    }

    /// 判断两个字符串是否不相等
    pub fn not_equals(a: &str, b: &str) -> bool {
        not_equals(a, b)
    }

    /// 获取当前时间戳（秒）
    pub fn now_timestamp() -> i64 {
        now_timestamp()
    }

    /// 获取当前时间戳（毫秒）
    pub fn now_timestamp_millis() -> i64 {
        now_timestamp_millis()
    }

    /// 在 URL 上拼接查询参数
    pub fn join_param(url: &str, param_str: &str) -> String {
        join_param(url, param_str)
    }

    /// 在 URL 上拼接单个查询参数
    pub fn join_param_kv(url: &str, key: &str, value: &str) -> String {
        join_param_kv(url, key, value)
    }

    /// 在 URL 上拼接锚点参数
    pub fn join_sharp_param(url: &str, param_str: &str) -> String {
        join_sharp_param(url, param_str)
    }

    /// 拼接两个 URL
    pub fn splice_two_url(url1: &str, url2: &str) -> String {
        splice_two_url(url1, url2)
    }

    /// 字符串模糊匹配（支持 `*` 通配符）
    pub fn vague_match(patt: &str, s: &str) -> bool {
        vague_match(patt, s)
    }

    /// 将日期格式化为 `yyyy-MM-dd HH:mm:ss`
    pub fn format_date_timestamp(secs: i64) -> String {
        format_date_timestamp(secs)
    }

    /// 指定毫秒后的时间（格式化）
    pub fn format_after_date(ms: i64) -> String {
        format_after_date(ms)
    }

    /// 时间戳 + 随机数拼接标记串
    pub fn get_marking28() -> String {
        get_marking28()
    }

    /// 生成 32 字符 UUID（无横线）
    pub fn random_uuid() -> String {
        random_uuid()
    }

    /// 判断是否为基本类型名
    pub fn is_basic_type(type_name: &str) -> bool {
        is_basic_type(type_name)
    }

    /// 逗号分隔字符串转列表
    pub fn convert_string_to_list(s: &str) -> Vec<String> {
        convert_string_to_list(s)
    }

    /// 列表转逗号分隔字符串
    pub fn convert_list_to_string(list: &[String]) -> String {
        convert_list_to_string(list)
    }

    /// URL 编码（UTF-8）
    pub fn encode_url(url: &str) -> String {
        encode_url(url)
    }

    /// URL 解码（UTF-8）
    pub fn decode_url(url: &str) -> String {
        decode_url(url)
    }

    /// 判断字符串是否为 URL
    pub fn is_url(s: &str) -> bool {
        is_url(s)
    }

    /// 将日志等级字符串转为 int
    pub fn translate_log_level_to_int(level: &str) -> i32 {
        translate_log_level_to_int(level)
    }

    /// 将日志等级 int 转为字符串
    pub fn translate_log_level_to_string(level: i32) -> String {
        translate_log_level_to_string(level)
    }

    /// list1 是否完全包含 list2
    pub fn list1_contain_list2_all(list1: &[String], list2: &[String]) -> bool {
        list1_contain_list2_all(list1, list2)
    }

    /// list1 是否包含 list2 任意元素
    pub fn list1_contain_list2_any(list1: &[String], list2: &[String]) -> bool {
        list1_contain_list2_any(list1, list2)
    }

    /// 从 list1 剔除 list2 元素（克隆副本）
    pub fn list1_remove_by_list2(list1: &[String], list2: &[String]) -> Vec<String> {
        list1_remove_by_list2(list1, list2)
    }

    /// 搜索集合并分页
    pub fn search_list(
        data_list: &[String],
        prefix: &str,
        keyword: &str,
        start: i32,
        size: i32,
        sort_type: bool,
    ) -> Vec<String> {
        search_list(data_list, prefix, keyword, start, size, sort_type)
    }

    /// 数组元素逗号拼接
    pub fn array_join(arr: &[&str]) -> String {
        array_join(arr)
    }

    /// 值转字符串，null 等价空串
    pub fn value_to_string(value: Option<&str>) -> String {
        value_to_string(value)
    }
}

/// 生成指定长度的随机字符串（字母+数字）
pub fn random_string(len: usize) -> String {
    let chars = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
    let mut rng = rand::rng();
    (0..len)
        .map(|_| chars[rng.random_range(0..chars.len())] as char)
        .collect()
}

/// 生成指定区间的随机整数（含边界）
pub fn random_number(min: i32, max: i32) -> i32 {
    let (lo, hi) = if min <= max { (min, max) } else { (max, min) };
    rand::rng().random_range(lo..=hi)
}

/// 判断字符串是否为空
pub fn is_empty(s: &str) -> bool {
    s.is_empty()
}

/// 判断字符串是否不为空
pub fn is_not_empty(s: &str) -> bool {
    !s.is_empty()
}

/// 是否包含不可打印 ASCII 字符（对应 Java `SaFoxUtil.hasNonPrintableASCII`）。
pub fn has_non_printable_ascii(s: &str) -> bool {
    s.chars().any(|c| {
        let code = c as u32;
        code <= 31 || code == 127
    })
}

/// 判断切片是否为空
pub fn is_empty_array<T>(arr: &[T]) -> bool {
    arr.is_empty()
}

/// 判断字符串列表是否为空
pub fn is_empty_list(list: &[String]) -> bool {
    list.is_empty()
}

/// 判断两个字符串是否相等
pub fn equals(a: &str, b: &str) -> bool {
    a == b
}

/// 判断两个字符串是否不相等
pub fn not_equals(a: &str, b: &str) -> bool {
    a != b
}

/// 获取当前时间戳（秒）
pub fn now_timestamp() -> i64 {
    Utc::now().timestamp()
}

/// 获取当前时间戳（毫秒）
pub fn now_timestamp_millis() -> i64 {
    Utc::now().timestamp_millis()
}

/// 在 URL 上拼接查询参数（对应 Java `joinParam(String url, String paramStr)`）
pub fn join_param(url: &str, param_str: &str) -> String {
    if param_str.is_empty() {
        return url.to_string();
    }
    let url = if url.is_empty() { "" } else { url };
    match url.rfind('?') {
        None => format!("{url}?{param_str}"),
        Some(idx) if idx + 1 == url.len() => format!("{url}{param_str}"),
        Some(_) => {
            let needs_amp = !url.ends_with('&') && !param_str.starts_with('&');
            if needs_amp {
                format!("{url}&{param_str}")
            } else {
                format!("{url}{param_str}")
            }
        }
    }
}

/// 在 URL 上拼接单个查询参数
pub fn join_param_kv(url: &str, key: &str, value: &str) -> String {
    if is_empty(url) || is_empty(key) {
        return url.to_string();
    }
    join_param(url, &format!("{key}={value}"))
}

/// 在 URL 上拼接锚点参数
pub fn join_sharp_param(url: &str, param_str: &str) -> String {
    if param_str.is_empty() {
        return url.to_string();
    }
    let url = if url.is_empty() { "" } else { url };
    match url.rfind('#') {
        None => format!("{url}#{param_str}"),
        Some(idx) if idx + 1 == url.len() => format!("{url}{param_str}"),
        Some(_) => {
            let needs_amp = !url.ends_with('&') && !param_str.starts_with('&');
            if needs_amp {
                format!("{url}&{param_str}")
            } else {
                format!("{url}{param_str}")
            }
        }
    }
}

/// 拼接两个 URL
pub fn splice_two_url(url1: &str, url2: &str) -> String {
    match (url1.is_empty(), url2.is_empty()) {
        (true, _) => url2.to_string(),
        (_, true) => url1.to_string(),
        _ if url2.starts_with("http") => url2.to_string(),
        _ => format!("{url1}{url2}"),
    }
}

/// 字符串模糊匹配（支持 `*` 通配符）
pub fn vague_match(patt: &str, s: &str) -> bool {
    if patt == s {
        return true;
    }
    if !patt.contains('*') {
        return patt == s;
    }
    vague_match_method(patt, s)
}

/// 通配符 DP 匹配
fn vague_match_method(pattern: &str, s: &str) -> bool {
    let p = pattern.as_bytes();
    let t = s.as_bytes();
    let m = t.len();
    let n = p.len();
    let mut dp = vec![vec![false; n + 1]; m + 1];
    dp[0][0] = true;
    for j in 1..=n {
        if p[j - 1] == b'*' {
            dp[0][j] = true;
        } else {
            break;
        }
    }
    for i in 1..=m {
        for j in 1..=n {
            if p[j - 1] == b'*' {
                dp[i][j] = dp[i][j - 1] || dp[i - 1][j];
            } else if t[i - 1] == p[j - 1] {
                dp[i][j] = dp[i - 1][j - 1];
            }
        }
    }
    dp[m][n]
}

/// 将秒级时间戳格式化为 `yyyy-MM-dd HH:mm:ss`
pub fn format_date_timestamp(secs: i64) -> String {
    Local
        .timestamp_opt(secs, 0)
        .single()
        .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
        .unwrap_or_default()
}

/// 指定毫秒后的时间（格式化）
pub fn format_after_date(ms: i64) -> String {
    let target = Utc::now().timestamp_millis() + ms;
    format_date_timestamp(target / 1000)
}

/// 时间戳 + 随机数拼接标记串
pub fn get_marking28() -> String {
    format!(
        "{}{}",
        now_timestamp_millis(),
        random_number(0, i32::MAX)
    )
}

/// 判断是否为基本类型名
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

/// 生成 32 字符 UUID（无横线）
pub fn random_uuid() -> String {
    let mut rng = rand::rng();
    let bytes: [u8; 16] = rng.random();
    hex_encode(&bytes)
}

/// hex 编码（小写）
fn hex_encode(bytes: &[u8]) -> String {
    const HEX: &[u8] = b"0123456789abcdef";
    let mut s = String::with_capacity(bytes.len() * 2);
    for &b in bytes {
        s.push(HEX[(b >> 4) as usize] as char);
        s.push(HEX[(b & 0x0f) as usize] as char);
    }
    s
}

/// 逗号分隔字符串转列表
pub fn convert_string_to_list(s: &str) -> Vec<String> {
    if is_empty(s) {
        return Vec::new();
    }
    s.split(',')
        .map(str::trim)
        .filter(|part| is_not_empty(part))
        .map(str::to_string)
        .collect()
}

/// 列表转逗号分隔字符串
pub fn convert_list_to_string(list: &[String]) -> String {
    if list.is_empty() {
        return String::new();
    }
    list.join(",")
}

/// 逗号分隔字符串转数组
pub fn convert_string_to_array(s: &str) -> Vec<String> {
    convert_string_to_list(s)
}

/// 数组转逗号分隔字符串
pub fn convert_array_to_string(arr: &[String]) -> String {
    convert_list_to_string(arr)
}

/// URL 编码（UTF-8）
pub fn encode_url(url: &str) -> String {
    urlencoding::encode(url).into_owned()
}

/// URL 解码（UTF-8）
pub fn decode_url(url: &str) -> String {
    urlencoding::decode(url)
        .map(|cow| cow.into_owned())
        .unwrap_or_else(|_| url.to_string())
}

/// 判断字符串是否为 URL
pub fn is_url(s: &str) -> bool {
    !is_empty(s) && URL_REGEX.is_match(s)
}

/// 将日志等级字符串转为 int
pub fn translate_log_level_to_int(level: &str) -> i32 {
    let idx = LOG_LEVEL_LIST
        .iter()
        .position(|&item| item == level)
        .unwrap_or(1) as i32;
    if idx <= 0 || idx >= LOG_LEVEL_LIST.len() as i32 {
        1
    } else {
        idx
    }
}

/// 将日志等级 int 转为字符串
pub fn translate_log_level_to_string(level: i32) -> String {
    let idx = if level <= 0 || level >= LOG_LEVEL_LIST.len() as i32 {
        1
    } else {
        level as usize
    };
    LOG_LEVEL_LIST[idx].to_string()
}

/// list1 是否完全包含 list2
pub fn list1_contain_list2_all(list1: &[String], list2: &[String]) -> bool {
    if list2.is_empty() {
        return true;
    }
    if list1.is_empty() {
        return false;
    }
    list2.iter().all(|item| list1.contains(item))
}

/// list1 是否包含 list2 任意元素
pub fn list1_contain_list2_any(list1: &[String], list2: &[String]) -> bool {
    if list1.is_empty() || list2.is_empty() {
        return false;
    }
    list2.iter().any(|item| list1.contains(item))
}

/// 从 list1 剔除 list2 元素（克隆副本）
pub fn list1_remove_by_list2(list1: &[String], list2: &[String]) -> Vec<String> {
    if list1.is_empty() || list2.is_empty() {
        return list1.to_vec();
    }
    list1
        .iter()
        .filter(|item| !list2.contains(item))
        .cloned()
        .collect()
}

/// 搜索集合并分页
pub fn search_list(
    data_list: &[String],
    prefix: &str,
    keyword: &str,
    start: i32,
    size: i32,
    sort_type: bool,
) -> Vec<String> {
    let prefix = if prefix.is_empty() { "" } else { prefix };
    let keyword = if keyword.is_empty() { "" } else { keyword };
    let mut list: Vec<String> = data_list
        .iter()
        .filter(|key| key.starts_with(prefix) && key.contains(keyword))
        .cloned()
        .collect();
    search_list_page(&mut list, start, size, sort_type)
}

/// 对已有列表分页
pub fn search_list_page(list: &mut [String], start: i32, size: i32, sort_type: bool) -> Vec<String> {
    let mut owned = list.to_vec();
    if !sort_type {
        owned.reverse();
    }
    let start = start.max(0) as usize;
    let end = if size < 0 {
        owned.len()
    } else {
        start.saturating_add(size as usize)
    };
    owned.into_iter().skip(start).take(end.saturating_sub(start)).collect()
}

/// 数组元素逗号拼接
pub fn array_join(arr: &[&str]) -> String {
    arr.join(",")
}

/// 值转字符串，None 等价空串
pub fn value_to_string(value: Option<&str>) -> String {
    value.unwrap_or("").to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn random_string_len() {
        let s = random_string(16);
        assert_eq!(s.len(), 16);
    }

    #[test]
    fn is_empty_and_equals() {
        assert!(is_empty(""));
        assert!(is_not_empty("x"));
        assert!(equals("a", "a"));
        assert!(not_equals("a", "b"));
    }

    #[test]
    fn join_param_url() {
        assert_eq!(
            join_param("http://a.com", "id=1"),
            "http://a.com?id=1"
        );
        assert_eq!(
            join_param("http://a.com?", "id=1"),
            "http://a.com?id=1"
        );
        assert_eq!(
            join_param("http://a.com?x=1", "id=2"),
            "http://a.com?x=1&id=2"
        );
    }

    #[test]
    fn splice_and_vague_match() {
        assert_eq!(splice_two_url("http://a.com", "/api"), "http://a.com/api");
        assert!(vague_match("user*", "user-add"));
        assert!(!vague_match("user*", "art-add"));
    }

    #[test]
    fn convert_and_list_ops() {
        assert_eq!(
            convert_string_to_list("a, b ,c"),
            vec!["a", "b", "c"]
        );
        assert!(list1_contain_list2_all(
            &["a".into(), "b".into()],
            &["a".into()]
        ));
    }

    #[test]
    fn encode_decode_url_roundtrip() {
        let raw = "hello world";
        assert_eq!(decode_url(&encode_url(raw)), raw);
    }
}
