//! 无参数、动态返回值函数端口。

use std::any::Any;

/// Java `Object` 返回值的线程安全 Rust 表示。
pub type SaAnyValue = Box<dyn Any + Send + Sync>;

/// Java `SaRetFunction` 的 Rust 对应 trait。
pub trait SaRetFunction: Send + Sync {
    /// 执行回调并返回动态值。
    fn run(&self) -> SaAnyValue;
}
