//! `SaCreateStpLogicFunction` —— 1:1 对应 Java `cn.dev33.satoken.fun.SaCreateStpLogicFunction`

pub trait SaCreateStpLogicFunction: Send + Sync + 'static {
    fn create(&self, login_type: &str) -> ();
}
