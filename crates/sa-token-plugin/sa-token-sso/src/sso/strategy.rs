//! Customizable SSO client and server policies.

pub mod sa_sso_client_strategy;
pub mod sa_sso_server_strategy;

pub use sa_sso_client_strategy::SaSsoClientStrategy;
pub use sa_sso_server_strategy::SaSsoServerStrategy;
