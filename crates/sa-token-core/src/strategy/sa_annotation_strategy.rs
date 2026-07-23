//! `SaAnnotationStrategy` —— 1:1 对应 Java `cn.dev33.satoken.strategy.SaAnnotationStrategy`
//!
//! 注解拦截策略（如何校验注解）。

use crate::fun::strategy::sa_check_element_annotation_function::SaCheckElementAnnotationFunction;
use crate::fun::strategy::sa_check_or_annotation_function::SaCheckOrAnnotationFunction;
use crate::fun::strategy::sa_get_annotation_function::SaGetAnnotationFunction;
use crate::fun::strategy::sa_is_annotation_present_function::SaIsAnnotationPresentFunction;
use std::sync::RwLock;

/// 注解策略
pub struct SaAnnotationStrategy {
    /// 「方法/类上是否存在该注解」的函数
    pub is_annotation_present: Box<dyn SaIsAnnotationPresentFunction>,
    /// 「获取注解」的函数
    pub get_annotation: Box<dyn SaGetAnnotationFunction>,
    /// 「校验元素注解」的函数
    pub check_element_annotation: Box<dyn SaCheckElementAnnotationFunction>,
    /// 「或分组校验」的函数
    pub check_or_annotation: Box<dyn SaCheckOrAnnotationFunction>,
}

impl SaAnnotationStrategy {
    pub fn new(
        is_annotation_present: Box<dyn SaIsAnnotationPresentFunction>,
        get_annotation: Box<dyn SaGetAnnotationFunction>,
        check_element_annotation: Box<dyn SaCheckElementAnnotationFunction>,
        check_or_annotation: Box<dyn SaCheckOrAnnotationFunction>,
    ) -> Self {
        Self {
            is_annotation_present,
            get_annotation,
            check_element_annotation,
            check_or_annotation,
        }
    }

    /// 默认策略实例（占位）
    pub fn default_impl() -> Self {
        // 占位实现
        struct TrueFn;
        impl crate::fun::strategy::sa_is_annotation_present_function::SaIsAnnotationPresentFunction
            for TrueFn
        {
            fn is_annotation_present(&self, _method: &(), _annot_cls: &str) -> bool {
                false
            }
        }
        struct NullGet;
        impl crate::fun::strategy::sa_get_annotation_function::SaGetAnnotationFunction for NullGet {
            fn get_annotation(&self, _method: &(), _annot_cls: &str) -> serde_json::Value {
                serde_json::Value::Null
            }
        }
        struct NullCheck;
        impl crate::fun::strategy::sa_check_element_annotation_function::SaCheckElementAnnotationFunction for NullCheck {
            fn check_element_annotation(&self, _method: &()) {}
        }
        struct NullOr;
        impl crate::fun::strategy::sa_check_or_annotation_function::SaCheckOrAnnotationFunction for NullOr {
            fn check_or_annotation(&self, _methods: &[()]) {}
        }
        Self::new(
            Box::new(TrueFn),
            Box::new(NullGet),
            Box::new(NullCheck),
            Box::new(NullOr),
        )
    }
}

/// 全局注解策略实例（OnceLock 单例）
pub fn annotation_strategy() -> &'static RwLock<Option<SaAnnotationStrategy>> {
    static INST: std::sync::OnceLock<RwLock<Option<SaAnnotationStrategy>>> =
        std::sync::OnceLock::new();
    INST.get_or_init(|| RwLock::new(None))
}
