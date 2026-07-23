//! `SaIsAnnotationPresentFunction` —— 1:1 对应 Java `cn.dev33.satoken.fun.SaIsAnnotationPresentFunction`

pub trait SaIsAnnotationPresentFunction: Send + Sync + 'static {
    fn is_annotation_present(&self, method: &(), annot_cls: &str) -> bool;
}
