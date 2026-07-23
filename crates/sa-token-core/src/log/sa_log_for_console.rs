//! 控制台日志实现（对应 Java `cn.dev33.satoken.log.SaLogForConsole`）。

use super::sa_log::{SaLog, SaLogLevel};

/// 日志级别常量（对应 Java `SaLogForConsole.trace/debug/info/warn/error/fatal`）
pub const TRACE: i32 = 1;
pub const DEBUG: i32 = 2;
pub const INFO: i32 = 3;
pub const WARN: i32 = 4;
pub const ERROR: i32 = 5;
pub const FATAL: i32 = 6;

/// 控制台日志实现
pub struct SaLogForConsole;

impl SaLog for SaLogForConsole {
    fn log(&self, level: SaLogLevel, tag: &str, message: &str) {
        match level {
            SaLogLevel::Trace => tracing::trace!("[{}] {}", tag, message),
            SaLogLevel::Debug => tracing::debug!("[{}] {}", tag, message),
            SaLogLevel::Info => tracing::info!("[{}] {}", tag, message),
            SaLogLevel::Warn => tracing::warn!("[{}] {}", tag, message),
            SaLogLevel::Error => tracing::error!("[{}] {}", tag, message),
            SaLogLevel::Fatal => tracing::error!("[{}] FATAL: {}", tag, message),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn console_log_levels() {
        let logger = SaLogForConsole;
        logger.trace("test", "trace");
        logger.info("test", "info");
        assert_eq!(TRACE, 1);
        assert_eq!(FATAL, 6);
    }
}
