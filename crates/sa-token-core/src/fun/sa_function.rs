//! 无参数、无返回值函数端口。

/// Java `SaFunction` 的 Rust 对应 trait。
pub trait SaFunction: Send + Sync {
    /// 执行回调。
    fn run(&self);
}

impl<F> SaFunction for F
where
    F: Fn() + Send + Sync,
{
    fn run(&self) {
        self();
    }
}
