//! `SaSugar` —— 1:1 对应 Java `cn.dev33.satoken.util.SaSugar`
//!
//! Java 中的语法糖集合；Rust 侧保留常用空值/集合辅助。

/// SaSugar 工具（对应 Java 静态方法集合）
pub struct SaSugar;

impl SaSugar {
    /// 返回空字符串
    pub fn empty_string() -> String {
        String::new()
    }

    /// 返回空字符串列表
    pub fn empty_list() -> Vec<String> {
        Vec::new()
    }

    /// 将可变参数转为列表
    pub fn to_list(items: &[&str]) -> Vec<String> {
        items.iter().map(|s| (*s).to_string()).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sugar_helpers() {
        assert!(SaSugar::empty_string().is_empty());
        assert!(SaSugar::empty_list().is_empty());
        assert_eq!(SaSugar::to_list(&["a", "b"]), vec!["a", "b"]);
    }
}
