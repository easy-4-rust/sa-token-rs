//! 日志接口（对应 Java `cn.dev33.satoken.log.SaLog`）。

/// 日志级别
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum SaLogLevel {
    /// TRACE
    Trace,
    /// DEBUG
    Debug,
    /// INFO
    Info,
    /// WARN
    Warn,
    /// ERROR
    Error,
    /// FATAL
    Fatal,
}

/// 日志 trait
pub trait SaLog: Send + Sync + 'static {
    /// 输出日志
    fn log(&self, level: SaLogLevel, tag: &str, message: &str);

    /// TRACE
    fn trace(&self, tag: &str, message: &str) {
        self.log(SaLogLevel::Trace, tag, message);
    }

    /// DEBUG
    fn debug(&self, tag: &str, message: &str) {
        self.log(SaLogLevel::Debug, tag, message);
    }

    /// INFO
    fn info(&self, tag: &str, message: &str) {
        self.log(SaLogLevel::Info, tag, message);
    }

    /// WARN
    fn warn(&self, tag: &str, message: &str) {
        self.log(SaLogLevel::Warn, tag, message);
    }

    /// ERROR
    fn error(&self, tag: &str, message: &str) {
        self.log(SaLogLevel::Error, tag, message);
    }

    /// FATAL
    fn fatal(&self, tag: &str, message: &str) {
        self.log(SaLogLevel::Fatal, tag, message);
    }
}

/// 控制台日志实现
pub struct SaLogForConsole;

impl SaLog for SaLogForConsole {
    fn log(&self, level: SaLogLevel, tag: &str, message: &str) {
        match level {
            SaLogLevel::Trace => tracing::trace!(tag = tag, message),
            SaLogLevel::Debug => tracing::debug!(tag = tag, message),
            SaLogLevel::Info => tracing::info!(tag = tag, message),
            SaLogLevel::Warn => tracing::warn!(tag = tag, message),
            SaLogLevel::Error => tracing::error!(tag = tag, message),
            SaLogLevel::Fatal => tracing::error!(tag = tag, message),
        }
    }
}
