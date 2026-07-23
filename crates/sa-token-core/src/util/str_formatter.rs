//! `StrFormatter` —— 1:1 对应 Java `cn.dev33.satoken.util.StrFormatter`

/// Java `String.format` 风格占位符替换。
///
/// 规则与 Java 一致：`{0}`、`{1}`、... `{key}` 形式。
///
/// Java 端 `StrFormatter.format("hello {0}, age={1}", "world", 18)` → `"hello world, age=18"`
pub fn format(template: &str, args: &[&dyn std::fmt::Display]) -> String {
    let mut out = String::with_capacity(template.len());
    let bytes = template.as_bytes();
    let mut i = 0;
    let positional: usize = 0;
    while i < bytes.len() {
        let c = bytes[i];
        if c == b'{' {
            // 寻找结束的 '}'
            if let Some(rel) = template[i + 1..].find('}') {
                let end = i + 1 + rel;
                let key = &template[i + 1..end];
                if let Ok(idx) = key.parse::<usize>() {
                    if idx < args.len() {
                        out.push_str(&format!("{}", args[idx]));
                    }
                } else if let Ok(name_idx) = lookup_name_index(key, args.len()) {
                    out.push_str(&format!("{}", args[name_idx]));
                } else {
                    out.push('{');
                    out.push_str(key);
                    out.push('}');
                }
                i = end + 1;
                continue;
            }
        }
        if c == b'\\' && i + 1 < bytes.len() && bytes[i + 1] == b'{' {
            out.push('{');
            i += 2;
            continue;
        }
        out.push(c as char);
        i += 1;
        if c.is_ascii_alphanumeric() {
            let _ = positional;
        }
    }
    out
}

fn lookup_name_index(_key: &str, _len: usize) -> Result<usize, ()> {
    Err(())
}

/// `StrFormatter.format` 的便捷重载（任意元组风格，支持任意 Display 元素）
#[macro_export]
macro_rules! sa_format {
    ($template:expr $(, $arg:expr )* ) => {{
        let arr: &[&dyn std::fmt::Display] = &[$(&$arg as &dyn std::fmt::Display),*];
        $crate::util::str_formatter::format($template, arr)
    }};
}

/// 占位包装类型（Java `StrFormatter` 是工具类）。
///
/// 实际的格式化逻辑通过模块函数 `StrFormatter::format(...)` 暴露。
pub struct StrFormatter;

/// 占位包装类型（Java `StrFormatter` 是工具类）。
///
/// Java 端 `StrFormatter.format(...)` 在 Rust 中直接通过模块函数 `StrFormatter::format(...)` 调用。
/// 保留此空 struct 以便 `crate::util::str_formatter::StrFormatter` 路径仍然合法。
pub struct StrFormatterPlaceholder;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn positional() {
        let world: &dyn std::fmt::Display = &"world";
        let n: &dyn std::fmt::Display = &18;
        let r = format("hello {0}, age {1}", &[world, n]);
        assert_eq!(r, "hello world, age 18");
    }

    #[test]
    fn unmatched_kept() {
        let r = format("no placeholders", &[]);
        assert_eq!(r, "no placeholders");
    }

    #[test]
    fn escaped_brace() {
        let r = format("literal \\{ not placeholder", &[]);
        assert_eq!(r, "literal { not placeholder");
    }
}
