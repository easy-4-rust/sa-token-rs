//! 响应抽象（对应 Java `cn.dev33.satoken.context.model.SaResponse`）。
use super::sa_cookie::SaCookie;

/// 响应抽象 trait
pub trait SaResponse: Send + Sync {
    /// 获取原始对象
    fn source(&self) -> &dyn std::any::Any;

    /// 设置响应状态码
    fn set_status(&self, sc: u16);

    /// 设置响应头
    fn set_header(&self, name: &str, value: &str);

    /// 添加响应头
    fn add_header(&self, name: &str, value: &str);

    /// 添加 Cookie
    fn add_cookie(&self, cookie: SaCookie);

    /// 删除 Cookie
    fn delete_cookie(&self, name: &str);

    /// 重定向
    fn redirect(&self, url: &str);
}
