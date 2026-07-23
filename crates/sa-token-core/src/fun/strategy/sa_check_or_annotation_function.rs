//! `SaCheckOrAnnotationFunction` —— 1:1 对应 Java `cn.dev33.satoken.fun.SaCheckOrAnnotationFunction`

pub trait SaCheckOrAnnotationFunction: Send + Sync + 'static {
    fn check_or_annotation(&self, methods: &[()]);
}
