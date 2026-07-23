//! `SaFirewallCheckFailHandleFunction` —— 1:1 对应 Java `cn.dev33.satoken.fun.strategy.SaFirewallCheckFailHandleFunction`
//!
//! 防火墙校验失败时的处理函数。

use crate::context::model::{sa_request::SaRequest, sa_response::SaResponse};
use crate::exception::SaTokenException;

/// 防火墙校验失败处理函数（对应 Java `SaFirewallCheckFailHandleFunction`）。
pub type SaFirewallCheckFailHandleFunction = Box<
    dyn Fn(&SaTokenException, &dyn SaRequest, &dyn SaResponse) + Send + Sync,
>;
