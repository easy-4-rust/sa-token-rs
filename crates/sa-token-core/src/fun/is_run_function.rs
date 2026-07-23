//! 条件执行辅助对象（对应 Java `cn.dev33.satoken.fun.IsRunFunction`）。

/// 根据固定条件选择执行正向或反向回调。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IsRunFunction {
    /// 是否执行正向回调。
    pub is_run: bool,
}

impl IsRunFunction {
    /// 创建条件执行对象。
    pub const fn new(is_run: bool) -> Self {
        Self { is_run }
    }

    /// 条件为 `true` 时执行回调。
    pub fn exe<F>(&self, function: F) -> &Self
    where
        F: FnOnce(),
    {
        if self.is_run {
            function();
        }
        self
    }

    /// 条件为 `false` 时执行回调。
    pub fn no_exe<F>(&self, function: F) -> &Self
    where
        F: FnOnce(),
    {
        if !self.is_run {
            function();
        }
        self
    }
}
