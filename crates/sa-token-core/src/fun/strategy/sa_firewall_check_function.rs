//! `SaFirewallCheckFunction` —— 1:1 对应 Java `cn.dev33.satoken.fun.strategy.SaFirewallCheckFunction`
//!
//! 函数式接口：防火墙校验函数。

use crate::context::model::{sa_request::SaRequest, sa_response::SaResponse};
use crate::exception::SaTokenException;

/// 防火墙校验函数（对应 Java `SaFirewallCheckFunction.execute`）。
pub type SaFirewallCheckFunction =
    Box<dyn Fn(&dyn SaRequest, &dyn SaResponse) -> Result<(), SaTokenException> + Send + Sync>;
