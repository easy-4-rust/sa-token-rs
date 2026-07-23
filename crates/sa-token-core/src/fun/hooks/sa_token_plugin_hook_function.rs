//! 插件 Hook 函数（对应 Java `cn.dev33.satoken.fun.hooks.SaTokenPluginHookFunction`）。

use std::any::Any;

/// 插件 Hook 函数签名：接受插件实例（`&dyn Any` 形式），无返回值。
///
/// Rust 通过注册时的 `TypeId` 保留 Java 泛型监听类型，再将实际插件以
/// `Any` 传给回调做安全向下转换。
pub type SaTokenPluginHookFunction = Box<dyn Fn(&dyn Any) + Send + Sync>;
