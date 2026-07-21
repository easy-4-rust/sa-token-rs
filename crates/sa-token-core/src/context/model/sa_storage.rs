//! 存储抽象（对应 Java `cn.dev33.satoken.context.model.SaStorage`）。

/// 存储抽象 trait（请求级临时存储）
pub trait SaStorage: Send + Sync {
    /// 获取原始对象
    fn source(&self) -> &dyn std::any::Any;

    /// 获取值
    fn get(&self, key: &str) -> Option<String>;

    /// 设置值
    fn set(&self, key: &str, value: &str);

    /// 删除值
    fn delete(&self, key: &str);
}
