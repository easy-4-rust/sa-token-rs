use std::sync::Arc;

use actix_web::web;
use sa_token_core::stp::AsyncStpUtil;

/// Registers an isolated async runtime on an Actix service config.
pub fn register_async_runtime(cfg: &mut web::ServiceConfig, util: Arc<AsyncStpUtil>) {
    cfg.app_data(web::Data::from(util));
}
