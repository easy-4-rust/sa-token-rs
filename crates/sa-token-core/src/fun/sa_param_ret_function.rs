//! 单参数、有返回值函数端口。

/// Java `SaParamRetFunction<T, R>` 的 Rust 对应 trait。
pub trait SaParamRetFunction<T, R>: Send + Sync {
    /// 使用指定参数执行回调并返回结果。
    fn run(&self, parameter: T) -> R;
}

impl<T, R, F> SaParamRetFunction<T, R> for F
where
    F: Fn(T) -> R + Send + Sync,
{
    fn run(&self, parameter: T) -> R {
        self(parameter)
    }
}
