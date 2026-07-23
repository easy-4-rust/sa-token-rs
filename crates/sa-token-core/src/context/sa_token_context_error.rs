//! 上下文异常辅助（对应 Java `SaTokenContextException` 抛出语义）。

use crate::error::SaErrorCode;
use crate::exception::SaTokenContextException;

/// 终止当前调用并抛出上下文异常（对应 Java unchecked `SaTokenContextException`）。
#[cold]
pub fn raise_context_exception(message: impl Into<String>, code: i32) -> ! {
    let err = SaTokenContextException::with_code(message, code);
    panic!("{}", err);
}

/// 未能获取有效上下文处理器（Java `SaTokenContextDefaultImpl.ERROR_MESSAGE` + `CODE_10001`）。
pub fn raise_invalid_context_handler() -> ! {
    raise_context_exception("未能获取有效的上下文处理器", SaErrorCode::CODE_10001);
}

/// 上下文尚未初始化（Java `SaTokenContextForThreadLocalStaff.getModelBox` + `CODE_10002`）。
pub fn raise_context_not_initialized() -> ! {
    raise_context_exception("SaTokenContext 上下文尚未初始化", SaErrorCode::CODE_10002);
}
