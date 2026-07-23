//! 请求抽象（对应 Java `cn.dev33.satoken.context.model.SaRequest`）。

/// 请求抽象 trait
pub trait SaRequest: Send + Sync {
    /// 获取原始对象
    fn source(&self) -> &dyn std::any::Any;

    /// 获取请求参数
    fn get_param(&self, name: &str) -> Option<String>;

    /// 获取请求参数，空值时返回默认值（对应 Java 默认方法）
    fn get_param_or_default(&self, name: &str, default_value: &str) -> String {
        self.get_param(name)
            .filter(|value| !value.is_empty())
            .unwrap_or_else(|| default_value.to_string())
    }

    /// 检测参数是否等于指定值（对应 Java `isParam`）
    fn is_param(&self, name: &str, value: &str) -> bool {
        self.get_param(name)
            .map(|param| !param.is_empty() && param == value)
            .unwrap_or(false)
    }

    /// 检测是否提供了指定参数（对应 Java `hasParam`）
    fn has_param(&self, name: &str) -> bool {
        self.get_param(name)
            .map(|value| !value.is_empty())
            .unwrap_or(false)
    }

    /// 获取请求头
    fn get_header(&self, name: &str) -> Option<String>;

    /// 获取 Cookie 值
    fn get_cookie_value(&self, name: &str) -> Option<String>;

    /// 获取请求路径
    fn get_request_path(&self) -> String;

    /// 获取请求 URL
    fn get_url(&self) -> String;

    /// 获取请求方法（GET/POST 等）
    fn get_method(&self) -> String;

    /// 获取主机名
    fn get_host(&self) -> String;

    /// 是否为 AJAX 请求
    fn is_ajax(&self) -> bool;

    /// 转发请求
    fn forward(&self, path: &str);
}
