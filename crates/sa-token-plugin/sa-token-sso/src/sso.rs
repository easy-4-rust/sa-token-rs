//! Java-compatible SSO module tree.

pub mod config;
pub mod error;
pub mod exception;
pub mod function;
pub mod message;
pub mod model;
pub mod name;
pub mod processor;
pub mod sa_sso_manager;
pub mod strategy;
pub mod template;
pub mod util;

pub use sa_sso_manager::SaSsoManager;
