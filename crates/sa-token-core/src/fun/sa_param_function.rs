//! 单参数、无返回值函数端口。

/// Java `SaParamFunction<T>` 的 Rust 对应 trait。
pub trait SaParamFunction<T>: Send + Sync {
    /// 使用指定参数执行回调。
    fn run(&self, parameter: T);
}

impl<T, F> SaParamFunction<T> for F
where
    F: Fn(T) + Send + Sync,
{
    fn run(&self, parameter: T) {
        self(parameter);
    }
}
