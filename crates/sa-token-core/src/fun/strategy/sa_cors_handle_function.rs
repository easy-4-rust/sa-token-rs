//! `SaCorsHandleFunction` —— 1:1 对应 Java `cn.dev33.satoken.fun.SaCorsHandleFunction`

pub trait SaCorsHandleFunction: Send + Sync + 'static {
    fn handle_cors(&self);
}
