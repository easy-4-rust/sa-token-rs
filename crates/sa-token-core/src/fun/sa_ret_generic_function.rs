//! 无参数、泛型返回值函数端口。

/// Java `SaRetGenericFunction<T>` 的 Rust 对应 trait。
pub trait SaRetGenericFunction<T>: Send + Sync {
    /// 执行回调并返回泛型值。
    fn run(&self) -> T;
}

impl<T, F> SaRetGenericFunction<T> for F
where
    F: Fn() -> T + Send + Sync,
{
    fn run(&self) -> T {
        self()
    }
}
