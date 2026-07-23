//! 路由匹配链对象（对应 Java `cn.dev33.satoken.router.SaRouterStaff`）。
//!
//! 通过链式调用构造一个"匹配 → 校验"流程。
//!
//! # 设计说明
//!
//! Java 版 `SaRouterStaff::stop()` 通过 `StopMatchException` 跳出 Auth 函数。
//! 在 Rust 中没有 try/catch 控制流，因此改为以下两种模式：
//!
//! - **立即执行模式**：`check(...)` 等方法在 `is_hit=true` 时直接执行回调。
//! - **Result 模式**：`try_back()` 返回 `Result<String, SaTokenException>`，
//!   由调用方（Filter / Layer）决定如何处理 `BackResult` 与 `StopMatch`。
//!
//! # 示例
//!
//! ```ignore
//! SaRouter::new_match()
//!     .match_paths(&["/admin/**"])
//!     .not_match(&["/admin/login"])
//!     .check(|| { /* 登录校验 */ });
//! ```

use super::sa_http_method::SaHttpMethod;
use super::sa_router::SaRouter;
use crate::exception::{SaResult, SaTokenException};

/// 路由匹配链
#[derive(Debug)]
pub struct SaRouterStaff {
    /// 是否命中
    is_hit: bool,
}

impl Default for SaRouterStaff {
    fn default() -> Self {
        Self::new()
    }
}

impl SaRouterStaff {
    /// 创建一个新的匹配链，初始 `is_hit = true`
    pub fn new() -> Self {
        Self { is_hit: true }
    }

    /// 是否命中
    pub fn is_hit(&self) -> bool {
        self.is_hit
    }

    /// 手动设置命中标记
    pub fn set_hit(mut self, hit: bool) -> Self {
        self.is_hit = hit;
        self
    }

    /// 重置为命中
    pub fn reset(mut self) -> Self {
        self.is_hit = true;
        self
    }

    // -------- path 匹配 --------

    /// 路径匹配（数组形式）
    pub fn match_paths(mut self, patterns: &[&str]) -> Self {
        if self.is_hit {
            self.is_hit = SaRouter::is_match_curr_any(patterns);
        }
        self
    }

    /// 路径排除
    pub fn not_match(mut self, patterns: &[&str]) -> Self {
        if self.is_hit {
            self.is_hit = !SaRouter::is_match_curr_any(patterns);
        }
        self
    }

    // -------- Method 匹配 --------

    /// HTTP 方法匹配（枚举）
    pub fn match_methods(mut self, methods: &[SaHttpMethod]) -> Self {
        if self.is_hit {
            self.is_hit = SaRouter::is_match_curr_method(methods);
        }
        self
    }

    /// HTTP 方法排除（枚举）
    pub fn not_match_methods(mut self, methods: &[SaHttpMethod]) -> Self {
        if self.is_hit {
            self.is_hit = !SaRouter::is_match_curr_method(methods);
        }
        self
    }

    /// HTTP 方法匹配（字符串数组）
    ///
    /// # Errors
    ///
    /// 任一方法名无效时返回 Java 兼容错误码 `10321`。
    pub fn match_method_strs(mut self, methods: &[&str]) -> SaResult<Self> {
        if self.is_hit {
            let methods = SaHttpMethod::parse_all(methods)?;
            self.is_hit = SaRouter::is_match_curr_method(&methods);
        }
        Ok(self)
    }

    /// HTTP 方法排除（字符串数组）
    ///
    /// # Errors
    ///
    /// 任一方法名无效时返回 Java 兼容错误码 `10321`。
    pub fn not_match_method_strs(mut self, methods: &[&str]) -> SaResult<Self> {
        if self.is_hit {
            let methods = SaHttpMethod::parse_all(methods)?;
            self.is_hit = !SaRouter::is_match_curr_method(&methods);
        }
        Ok(self)
    }

    // -------- boolean / 谓词匹配 --------

    /// 根据 boolean 决定是否命中
    pub fn match_flag(mut self, flag: bool) -> Self {
        if self.is_hit {
            self.is_hit = flag;
        }
        self
    }

    /// 根据 boolean 决定是否排除
    pub fn not_match_flag(mut self, flag: bool) -> Self {
        if self.is_hit {
            self.is_hit = !flag;
        }
        self
    }

    /// 根据自定义谓词判断
    pub fn match_predicate<F>(mut self, f: F) -> Self
    where
        F: FnOnce(&SaRouterStaff) -> bool,
    {
        if self.is_hit {
            self.is_hit = f(&self);
        }
        self
    }

    /// 自定义谓词排除
    pub fn not_match_predicate<F>(mut self, f: F) -> Self
    where
        F: FnOnce(&SaRouterStaff) -> bool,
    {
        if self.is_hit {
            self.is_hit = !f(&self);
        }
        self
    }

    // -------- check 执行 --------

    /// 执行无参校验函数
    pub fn check<F>(self, f: F) -> Self
    where
        F: FnOnce(),
    {
        if self.is_hit {
            f();
        }
        self
    }

    /// 执行带参校验函数
    pub fn check_with<F>(self, f: F) -> Self
    where
        F: FnOnce(&SaRouterStaff),
    {
        if self.is_hit {
            f(&self);
        }
        self
    }

    /// 自由匹配：在 `free` 作用域中 [`SaTokenException::StopMatch`] 只跳出当前块。
    ///
    /// `BackResult` 与业务错误不会被吞掉，而是继续交给 adapter 处理。
    ///
    /// # Errors
    ///
    /// 回调返回除 `StopMatch` 外的错误时原样传播。
    pub fn free<F>(self, f: F) -> SaResult<Self>
    where
        F: FnOnce(&SaRouterStaff) -> SaResult<()>,
    {
        if self.is_hit {
            match f(&self) {
                Ok(()) | Err(SaTokenException::StopMatch) => {}
                Err(error) => return Err(error),
            }
        }
        Ok(self)
    }

    // -------- 提前退出（Result 风格） --------

    /// 返回 Result：若 is_hit 为 true 则产生 StopMatch 异常；否则 Ok(())
    pub fn stop(self) -> SaResult<()> {
        if self.is_hit {
            Err(SaTokenException::StopMatch)
        } else {
            Ok(())
        }
    }

    /// 返回结果：若 is_hit 为 true 则产生 BackResult 异常；否则 Ok(())
    pub fn back(self) -> SaResult<()> {
        if self.is_hit {
            Err(SaTokenException::BackResult {
                result: String::new(),
            })
        } else {
            Ok(())
        }
    }

    /// 带结果 back
    pub fn back_with(self, result: impl Into<String>) -> SaResult<()> {
        if self.is_hit {
            Err(SaTokenException::BackResult {
                result: result.into(),
            })
        } else {
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chain_short_circuits() {
        let staff = SaRouterStaff::new()
            .match_flag(false)
            .match_paths(&["/admin/**"]);
        assert!(!staff.is_hit());
    }

    #[test]
    fn chain_preserves_when_already_false() {
        let staff = SaRouterStaff::new()
            .match_flag(false)
            .match_paths(&["/admin/**"]);
        assert!(!staff.is_hit());
    }

    #[test]
    fn stop_returns_err_when_hit() {
        let staff = SaRouterStaff::new();
        assert!(matches!(staff.stop(), Err(SaTokenException::StopMatch)));
    }

    #[test]
    fn stop_returns_ok_when_not_hit() {
        let staff = SaRouterStaff::new().set_hit(false);
        assert!(staff.stop().is_ok());
    }

    #[test]
    fn lazy_predicate_is_not_evaluated_after_a_miss() {
        let mut evaluated = false;
        let staff = SaRouterStaff::new().match_flag(false).match_predicate(|_| {
            evaluated = true;
            true
        });
        assert!(!staff.is_hit());
        assert!(!evaluated);
    }

    #[test]
    fn free_catches_only_stop_match() {
        let staff = SaRouterStaff::new()
            .free(|_| Err(SaTokenException::StopMatch))
            .expect("stop only exits the free block");
        assert!(staff.is_hit());

        let error = SaRouterStaff::new()
            .free(|_| {
                Err(SaTokenException::BackResult {
                    result: "denied".to_owned(),
                })
            })
            .expect_err("back must escape the free block");
        assert_eq!(
            error,
            SaTokenException::BackResult {
                result: "denied".to_owned()
            }
        );
    }
}
