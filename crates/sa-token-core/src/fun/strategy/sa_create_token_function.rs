//! CreateToken 函数（对应 Java `cn.dev33.satoken.fun.strategy.SaCreateTokenFunction`）。

/// 生成 Token 字符串的函数
pub type SaCreateTokenFunction = Box<dyn Fn() -> String + Send + Sync>;
