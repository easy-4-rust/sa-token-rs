//! GenerateUniqueToken 函数（对应 Java `cn.dev33.satoken.fun.strategy.SaGenerateUniqueTokenFunction`）。

/// 生成唯一 Token 的函数
pub type SaGenerateUniqueTokenFunction = Box<dyn Fn(&str, &str) -> String + Send + Sync>;
