//! 请求抽象（对应 Java `cn.dev33.satoken.context.model.SaRequest`）。

/// 请求抽象 trait
pub trait SaRequest: Send + Sync {
    /// 获取原始对象
    fn source(&self) -> &dyn std::any::Any;

    /// 获取请求参数
    fn get_param(&self, name: &str) -> Option<String>;

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
