//! `SaGetAnnotationFunction` —— 1:1 对应 Java `cn.dev33.satoken.fun.SaGetAnnotationFunction`

pub trait SaGetAnnotationFunction: Send + Sync + 'static {
    fn get_annotation(&self, method: &(), annot_cls: &str) -> serde_json::Value;
}
