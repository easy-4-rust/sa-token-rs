//! CreateSession 函数（对应 Java `cn.dev33.satoken.fun.strategy.SaCreateSessionFunction`）。

use std::sync::Arc;

use crate::session::sa_session::SaSession;

/// Session 工厂函数
pub type SaCreateSessionFunction = Box<dyn Fn(&str) -> Arc<SaSession> + Send + Sync>;
