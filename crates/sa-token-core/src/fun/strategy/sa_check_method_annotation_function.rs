//! `SaCheckMethodAnnotationFunction` —— 1:1 对应 Java `cn.dev33.satoken.fun.SaCheckMethodAnnotationFunction`

pub trait SaCheckMethodAnnotationFunction: Send + Sync + 'static {
    fn check_method_annotation(&self, method: &());
}
