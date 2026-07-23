//! SaTokenPluginHookModel：插件 Hook 模型（对应 Java `cn.dev33.satoken.plugin.SaTokenPluginHookModel`）。

use std::any::TypeId;

use crate::fun::hooks::sa_token_plugin_hook_function::SaTokenPluginHookFunction;

/// Hook 模型：监听插件类型 + 执行函数
pub struct SaTokenPluginHookModel {
    /// 监听的插件 `TypeId`
    pub listener_type_id: TypeId,
    /// 监听插件类型的字符串名（用于匹配 `TypeId` 在跨 crate 时的可读性）
    pub listener_type_name: &'static str,
    /// 执行函数
    pub execute_function: SaTokenPluginHookFunction,
}

impl SaTokenPluginHookModel {
    /// 构造 Hook 模型
    pub fn new<T: Send + Sync + 'static>(execute_function: SaTokenPluginHookFunction) -> Self {
        Self {
            listener_type_id: TypeId::of::<T>(),
            listener_type_name: std::any::type_name::<T>(),
            execute_function,
        }
    }
}
