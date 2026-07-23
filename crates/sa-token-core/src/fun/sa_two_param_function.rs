//! 双参数、无返回值函数端口。

/// Java `SaTwoParamFunction<T, T2>` 的 Rust 对应 trait。
pub trait SaTwoParamFunction<T, T2>: Send + Sync {
    /// 使用两个参数执行回调。
    fn run(&self, first: T, second: T2);
}

impl<T, T2, F> SaTwoParamFunction<T, T2> for F
where
    F: Fn(T, T2) + Send + Sync,
{
    fn run(&self, first: T, second: T2) {
        self(first, second);
    }
}
