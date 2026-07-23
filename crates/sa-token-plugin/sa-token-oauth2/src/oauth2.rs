//! Java-compatible OAuth2 module tree.

pub mod annotation;
pub mod config;
pub mod consts;
pub mod dao;
pub mod data;
pub mod error;
pub mod exception;
pub mod function;
pub mod grant_type;
pub mod processor;
pub mod sa_oauth2_manager;
pub mod scope;
pub mod strategy;
pub mod template;

pub use sa_oauth2_manager::{SaOAuth2Manager, SaOAuth2Runtime};
