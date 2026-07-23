//! `SaCheckElementAnnotationFunction` —— 1:1 对应 Java `cn.dev33.satoken.fun.SaCheckElementAnnotationFunction`

/// 校验函数对象（针对 method/element）
pub trait SaCheckElementAnnotationFunction: Send + Sync + 'static {
    fn check_element_annotation(&self, method: &());
}
