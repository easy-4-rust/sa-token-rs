//! 全局异常说明（对应 Java `com.pj.test.GlobalException`）。
//!
//! Axum 侧错误在 handler 内转为 `AjaxJson`；本模块保留迁移对照。

/// 全局异常处理占位类型（对应 Java `@ControllerAdvice` 类）。
#[allow(dead_code)]
pub struct GlobalException;

impl GlobalException {
    /// 将错误消息格式化为前端可读文案（对应 Java `handlerException` 语义）。
    #[allow(dead_code)]
    pub fn format_message(err: &str) -> String {
        format!("系统异常: {err}")
    }
}
