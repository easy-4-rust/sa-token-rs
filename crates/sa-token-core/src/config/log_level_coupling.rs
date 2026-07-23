//! `SaFoxUtil.translateLogLevel*` 语义迁移。
//!
//! Java 端定义：
//! ```java
//! public static List<String> logLevelList = Arrays.asList("", "trace", "debug", "info", "warn", "error", "fatal");
//!
//! public static int translateLogLevelToInt(String level) {
//!     int levelInt = logLevelList.indexOf(level);
//!     if (levelInt <= 0 || levelInt >= logLevelList.size()) {
//!         levelInt = 1;
//!     }
//!     return levelInt;
//! }
//!
//! public static String translateLogLevelToString(int level) {
//!     if (level <= 0 || level >= logLevelList.size()) {
//!         level = 1;
//!     }
//!     return logLevelList.get(level);
//! }
//! ```
//!
//! 与 `SaTokenConfig::set_log_level` / `set_log_level_int` 配合使用，
//! 以保证 `log_level` 与 `log_level_int` 在赋值后同步。

/// 与 Java `logLevelList` 一一对应的常量表。索引 0 是空字符串，
/// 索引 1..=6 分别对应 `trace / debug / info / warn / error / fatal`。
const LOG_LEVEL_LIST: &[&str] = &["", "trace", "debug", "info", "warn", "error", "fatal"];

/// Java `SaFoxUtil.translateLogLevelToInt`。
///
/// | 输入            | 输出 |
/// | ---------------- | ---- |
/// | `"trace"`        | 1    |
/// | `"debug"`        | 2    |
/// | `"info"`         | 3    |
/// | `"warn"`         | 4    |
/// | `"error"`        | 5    |
/// | `"fatal"`        | 6    |
/// | `""` 或 任意其它 | 1    |
pub fn translate_log_level_to_int(level: &str) -> i32 {
    let level_int = LOG_LEVEL_LIST.iter().position(|v| *v == level);
    match level_int {
        Some(i) if i > 0 && i < LOG_LEVEL_LIST.len() => i as i32,
        _ => 1,
    }
}

/// Java `SaFoxUtil.translateLogLevelToString`。
///
/// | 输入              | 输出    |
/// | ------------------ | ------- |
/// | `1`                | `trace` |
/// | `2`                | `debug` |
/// | `3`                | `info`  |
/// | `4`                | `warn`  |
/// | `5`                | `error` |
/// | `6`                | `fatal` |
/// | `<=0` 或 `>=7`     | `trace` |
pub fn translate_log_level_to_string(level_int: i32) -> &'static str {
    let level = if level_int <= 0 || (level_int as usize) >= LOG_LEVEL_LIST.len() {
        1
    } else {
        level_int
    };
    LOG_LEVEL_LIST[level as usize]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn int_round_trip_matches_java() {
        // Forward direction (String -> int) per Java semantics.
        assert_eq!(translate_log_level_to_int("trace"), 1);
        assert_eq!(translate_log_level_to_int("debug"), 2);
        assert_eq!(translate_log_level_to_int("info"), 3);
        assert_eq!(translate_log_level_to_int("warn"), 4);
        assert_eq!(translate_log_level_to_int("error"), 5);
        assert_eq!(translate_log_level_to_int("fatal"), 6);
    }

    #[test]
    fn int_out_of_range_returns_one() {
        // Java: empty string is at index 0, which fails `levelInt <= 0` -> 1
        assert_eq!(translate_log_level_to_int(""), 1);
        // unknown name falls back to 1
        assert_eq!(translate_log_level_to_int("verbose"), 1);
        assert_eq!(translate_log_level_to_int("nonsense"), 1);
    }

    #[test]
    fn string_round_trip_matches_java() {
        assert_eq!(translate_log_level_to_string(1), "trace");
        assert_eq!(translate_log_level_to_string(2), "debug");
        assert_eq!(translate_log_level_to_string(3), "info");
        assert_eq!(translate_log_level_to_string(4), "warn");
        assert_eq!(translate_log_level_to_string(5), "error");
        assert_eq!(translate_log_level_to_string(6), "fatal");
    }

    #[test]
    fn string_out_of_range_returns_trace() {
        assert_eq!(translate_log_level_to_string(0), "trace");
        assert_eq!(translate_log_level_to_string(7), "trace");
        assert_eq!(translate_log_level_to_string(-1), "trace");
        assert_eq!(translate_log_level_to_string(99), "trace");
    }
}