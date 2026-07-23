//! `SaAutoRenewFunction` —— 1:1 对应 Java `cn.dev33.satoken.fun.SaAutoRenewFunction`

pub trait SaAutoRenewFunction: Send + Sync + 'static {
    /// 续签至指定 timeout (秒)，返回是否续签成功
    fn auto_renew(&self, login_id: &str, login_type: &str, timeout: i64) -> bool;
}
