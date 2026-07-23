//! SaTokenPlugin：插件总接口（对应 Java `cn.dev33.satoken.plugin.SaTokenPlugin`）。

/// Sa-Token 插件接口
pub trait SaTokenPlugin: Send + Sync + 'static {
    /// 安装插件
    fn install(&self);

    /// 卸载插件
    fn destroy(&self) {}

    /// 反射为 `&dyn Any`（用于插件管理器的 TypeId 比较）
    fn as_any(&self) -> &dyn std::any::Any;
}
