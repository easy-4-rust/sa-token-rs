//! SaTokenPluginHolder：插件管理器（对应 Java `cn.dev33.satoken.plugin.SaTokenPluginHolder`）。

use std::any::TypeId;

use crate::exception::{SaResult, SaTokenException};
use crate::fun::hooks::SaPluginHookFn as SaTokenPluginHookFunction;
use crate::plugin::sa_token_plugin::SaTokenPlugin;
use crate::plugin::sa_token_plugin_hook_model::SaTokenPluginHookModel;

/// Sa-Token 插件管理器
pub struct SaTokenPluginHolder {
    pub is_loaded: bool,
    pub spi_dir: &'static str,
    plugins: Vec<Box<dyn SaTokenPlugin>>,
    install_hooks: Vec<SaTokenPluginHookModel>,
    before_install_hooks: Vec<SaTokenPluginHookModel>,
    after_install_hooks: Vec<SaTokenPluginHookModel>,
    destroy_hooks: Vec<SaTokenPluginHookModel>,
    before_destroy_hooks: Vec<SaTokenPluginHookModel>,
    after_destroy_hooks: Vec<SaTokenPluginHookModel>,
}

impl Default for SaTokenPluginHolder {
    fn default() -> Self {
        Self::new()
    }
}

impl SaTokenPluginHolder {
    pub fn new() -> Self {
        Self {
            is_loaded: false,
            spi_dir: "satoken",
            plugins: Vec::new(),
            install_hooks: Vec::new(),
            before_install_hooks: Vec::new(),
            after_install_hooks: Vec::new(),
            destroy_hooks: Vec::new(),
            before_destroy_hooks: Vec::new(),
            after_destroy_hooks: Vec::new(),
        }
    }

    /// Marks static Rust plugin registration as initialized.
    pub fn init(&mut self) {
        if !self.is_loaded {
            self.is_loaded = true;
        }
    }

    /// 是否已安装指定类型插件
    pub fn is_installed<T: SaTokenPlugin>(&self) -> bool {
        let id = TypeId::of::<T>();
        self.plugins.iter().any(|p| p.as_any().type_id() == id)
    }

    /// Returns the installed plugin of the requested concrete type.
    pub fn get_plugin<T: SaTokenPlugin>(&self) -> Option<&T> {
        self.plugins
            .iter()
            .find_map(|plugin| plugin.as_any().downcast_ref::<T>())
    }

    /// 安装插件实例
    ///
    /// # Errors
    ///
    /// Returns a plugin error when the same concrete type is already installed.
    pub fn install_plugin(&mut self, plugin: Box<dyn SaTokenPlugin>) -> SaResult<&mut Self> {
        let plugin_type_id = plugin.as_any().type_id();
        if self
            .plugins
            .iter()
            .any(|installed| installed.as_any().type_id() == plugin_type_id)
        {
            return Err(plugin_error("插件已安装，不可重复安装"));
        }
        let plugin_ref = plugin.as_ref();
        Self::consume_hooks_inplace(&mut self.before_install_hooks, plugin_ref);
        let consume_count = Self::consume_hooks_inplace(&mut self.install_hooks, plugin_ref);
        if consume_count == 0 {
            plugin.install();
        }
        Self::consume_hooks_inplace(&mut self.after_install_hooks, plugin_ref);
        self.plugins.push(plugin);
        Ok(self)
    }

    /// 卸载插件
    ///
    /// # Errors
    ///
    /// Returns a plugin error when no plugin of that concrete type is installed.
    pub fn destroy_plugin(&mut self, plugin: &dyn SaTokenPlugin) -> SaResult<&mut Self> {
        let plugin_type_id = plugin.as_any().type_id();
        if !self
            .plugins
            .iter()
            .any(|installed| installed.as_any().type_id() == plugin_type_id)
        {
            return Err(plugin_error("插件未安装，无法卸载"));
        }
        Self::consume_hooks_inplace(&mut self.before_destroy_hooks, plugin);
        let consume_count = Self::consume_hooks_inplace(&mut self.destroy_hooks, plugin);
        if consume_count == 0 {
            plugin.destroy();
        }
        Self::consume_hooks_inplace(&mut self.after_destroy_hooks, plugin);
        Ok(self)
    }

    /// 复制插件列表
    pub fn get_plugin_list_copy(&self) -> Vec<&dyn SaTokenPlugin> {
        self.plugins.iter().map(|p| p.as_ref()).collect()
    }

    /// 注册 Install 钩子
    pub fn on_install<T: SaTokenPlugin + 'static>(
        &mut self,
        execute_function: SaTokenPluginHookFunction,
    ) -> SaResult<&mut Self> {
        if self.is_installed::<T>() {
            return Err(plugin_error(
                "插件已安装完毕，不允许再注册 Install 钩子函数",
            ));
        }
        self.install_hooks
            .push(SaTokenPluginHookModel::new::<T>(execute_function));
        Ok(self)
    }

    /// 注册 Install 前置钩子
    pub fn on_before_install<T: SaTokenPlugin + 'static>(
        &mut self,
        execute_function: SaTokenPluginHookFunction,
    ) -> SaResult<&mut Self> {
        if self.is_installed::<T>() {
            return Err(plugin_error(
                "插件已安装完毕，不允许再注册 Install 前置钩子函数",
            ));
        }
        self.before_install_hooks
            .push(SaTokenPluginHookModel::new::<T>(execute_function));
        Ok(self)
    }

    /// 注册 Install 后置钩子
    pub fn on_after_install<T: SaTokenPlugin + 'static>(
        &mut self,
        execute_function: SaTokenPluginHookFunction,
    ) -> &mut Self {
        if let Some(plugin) = self.get_plugin::<T>() {
            execute_function(plugin.as_any());
            return self;
        }
        self.after_install_hooks
            .push(SaTokenPluginHookModel::new::<T>(execute_function));
        self
    }

    /// 注册 Destroy 钩子
    pub fn on_destroy<T: SaTokenPlugin + 'static>(
        &mut self,
        execute_function: SaTokenPluginHookFunction,
    ) -> &mut Self {
        self.destroy_hooks
            .push(SaTokenPluginHookModel::new::<T>(execute_function));
        self
    }

    /// 注册 Destroy 前置钩子
    pub fn on_before_destroy<T: SaTokenPlugin + 'static>(
        &mut self,
        execute_function: SaTokenPluginHookFunction,
    ) -> &mut Self {
        self.before_destroy_hooks
            .push(SaTokenPluginHookModel::new::<T>(execute_function));
        self
    }

    /// 注册 Destroy 后置钩子
    pub fn on_after_destroy<T: SaTokenPlugin + 'static>(
        &mut self,
        execute_function: SaTokenPluginHookFunction,
    ) -> &mut Self {
        self.after_destroy_hooks
            .push(SaTokenPluginHookModel::new::<T>(execute_function));
        self
    }

    fn consume_hooks_inplace(
        hooks: &mut Vec<SaTokenPluginHookModel>,
        plugin: &dyn SaTokenPlugin,
    ) -> usize {
        let any = plugin.as_any();
        let mut count = 0;
        let mut i = 0;
        while i < hooks.len() {
            if hooks[i].listener_type_id == any.type_id() {
                let f = &hooks[i].execute_function;
                f(any);
                hooks.remove(i);
                count += 1;
            } else {
                i += 1;
            }
        }
        count
    }
}

fn plugin_error(message: impl Into<String>) -> SaTokenException {
    SaTokenException::Plugin {
        message: message.into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    struct MyPlugin {
        installed: &'static AtomicUsize,
    }

    impl SaTokenPlugin for MyPlugin {
        fn install(&self) {
            self.installed.fetch_add(1, Ordering::SeqCst);
        }

        fn as_any(&self) -> &dyn std::any::Any {
            self
        }
    }

    #[test]
    fn install_plugin_runs_install() {
        static COUNTER: AtomicUsize = AtomicUsize::new(0);
        COUNTER.store(0, Ordering::SeqCst);
        let mut holder = SaTokenPluginHolder::new();
        let p: Box<dyn SaTokenPlugin> = Box::new(MyPlugin {
            installed: &COUNTER,
        });
        holder.install_plugin(p).expect("plugin installation");
        assert_eq!(COUNTER.load(Ordering::SeqCst), 1);
        assert!(holder.is_installed::<MyPlugin>());
    }

    #[test]
    fn on_install_hook_replaces_default_install() {
        static COUNTER: AtomicUsize = AtomicUsize::new(0);
        COUNTER.store(0, Ordering::SeqCst);
        let mut holder = SaTokenPluginHolder::new();
        holder
            .on_install::<MyPlugin>(Box::new(|_p| {
                COUNTER.fetch_add(10, Ordering::SeqCst);
            }))
            .expect("register install hook");
        let p: Box<dyn SaTokenPlugin> = Box::new(MyPlugin {
            installed: &COUNTER,
        });
        holder.install_plugin(p).expect("hooked installation");
        assert_eq!(COUNTER.load(Ordering::SeqCst), 10);
    }

    #[test]
    fn lifecycle_rejects_duplicates_consumes_hooks_and_runs_late_after_install() {
        static COUNTER: AtomicUsize = AtomicUsize::new(0);
        COUNTER.store(0, Ordering::SeqCst);
        let mut holder = SaTokenPluginHolder::new();
        holder.init();
        holder.init();
        assert!(holder.is_loaded);

        holder
            .on_before_install::<MyPlugin>(Box::new(|_| {
                COUNTER.fetch_add(10, Ordering::SeqCst);
            }))
            .expect("before-install hook");
        holder.on_after_install::<MyPlugin>(Box::new(|_| {
            COUNTER.fetch_add(100, Ordering::SeqCst);
        }));
        holder
            .install_plugin(Box::new(MyPlugin {
                installed: &COUNTER,
            }))
            .expect("plugin installation");
        assert_eq!(COUNTER.load(Ordering::SeqCst), 111);
        assert!(holder.get_plugin::<MyPlugin>().is_some());

        holder.on_after_install::<MyPlugin>(Box::new(|_| {
            COUNTER.fetch_add(1_000, Ordering::SeqCst);
        }));
        assert_eq!(COUNTER.load(Ordering::SeqCst), 1_111);
        assert!(holder.on_install::<MyPlugin>(Box::new(|_| {})).is_err());
        assert!(
            holder
                .install_plugin(Box::new(MyPlugin {
                    installed: &COUNTER,
                }))
                .is_err()
        );

        let external = MyPlugin {
            installed: &COUNTER,
        };
        holder
            .destroy_plugin(&external)
            .expect("installed type can be destroyed");
        let missing = struct_plugin();
        assert!(holder.destroy_plugin(&missing).is_err());
    }

    struct OtherPlugin;

    impl SaTokenPlugin for OtherPlugin {
        fn install(&self) {}

        fn as_any(&self) -> &dyn std::any::Any {
            self
        }
    }

    fn struct_plugin() -> OtherPlugin {
        OtherPlugin
    }
}
