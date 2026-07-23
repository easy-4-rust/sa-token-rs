//! Web integration mapping for Java `SaTokenContextRegister`.
//! Responsibility is implemented by the `actix` adapter instead of Spring/Servlet crates.
use std::sync::Arc;
use actix_web::web;
use sa_token_core::stp::AsyncStpUtil;

/// Actix app wiring helper replacing Spring bean registration.
pub fn register_util(cfg: &mut web::ServiceConfig, util: web::Data<AsyncStpUtil>) {
    cfg.app_data(util);
}
