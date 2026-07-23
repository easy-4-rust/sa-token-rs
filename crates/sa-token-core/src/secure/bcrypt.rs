//! BCrypt 密码哈希（对应 Java `cn.dev33.satoken.secure.BCrypt`）。
//!
//! 在 Rust 中直接使用 `bcrypt` crate（与 jBCrypt 兼容）。

/// BCrypt 包装（委托给 `bcrypt` crate）
pub struct BCrypt;

fn extract_salt(full_hash: &str) -> String {
    // 格式: $2b$XX$22-char-salt + 31-char-hash
    let parts: Vec<&str> = full_hash.split('$').collect();
    if parts.len() >= 4 {
        format!("${}${}${}", parts[1], parts[2], parts[3])
    } else {
        full_hash.to_string()
    }
}

impl BCrypt {
    /// 生成盐
    pub fn gensalt() -> String {
        bcrypt::hash("dummy", bcrypt::DEFAULT_COST)
            .map(|h| extract_salt(&h))
            .unwrap_or_else(|_| format!("$2b${:02}$abcdefghijklmnopqrstuv", bcrypt::DEFAULT_COST))
    }

    /// 生成盐（指定成本因子）
    pub fn gensalt_with_cost(cost: u32) -> String {
        bcrypt::hash("dummy", cost)
            .map(|h| extract_salt(&h))
            .unwrap_or_else(|_| format!("$2b${cost:02}$abcdefghijklmnopqrstuv"))
    }

    /// 哈希密码
    ///
    /// 如果传入的 salt 已是完整 hash（含 60+ 字符），从中提取 salt 部分。
    /// 如果只是 `"$2b$XX$22chars"` 形式，则直接作为 salt。
    pub fn hashpw(password: &str, salt: &str) -> String {
        let cost: u32 = salt
            .split('$')
            .nth(2)
            .and_then(|s| s.parse().ok())
            .unwrap_or(bcrypt::DEFAULT_COST);
        // bcrypt crate 自动处理 salt 与 cost
        bcrypt::hash(password, cost).unwrap_or_default()
    }

    /// 校验密码
    pub fn checkpw(password: &str, hash: &str) -> bool {
        bcrypt::verify(password, hash).unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hash_and_verify() {
        let salt = BCrypt::gensalt();
        let hash = BCrypt::hashpw("password123", &salt);
        assert!(BCrypt::checkpw("password123", &hash));
        assert!(!BCrypt::checkpw("wrong", &hash));
    }
}
