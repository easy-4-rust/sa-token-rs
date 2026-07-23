//! Common and role-specific SSO protocol templates.

pub mod sa_sso_client_template;
pub mod sa_sso_client_util;
pub mod sa_sso_server_template;
pub mod sa_sso_server_util;
pub mod sa_sso_template;
pub mod sa_sso_util;

pub use sa_sso_client_template::{SaSsoClientLogoutFunction, SaSsoClientTemplate};
pub use sa_sso_client_util::SaSsoClientUtil;
pub use sa_sso_server_template::{SaSsoServerAuth, SaSsoServerTemplate};
pub use sa_sso_server_util::SaSsoServerUtil;
pub use sa_sso_template::SaSsoTemplate;
#[allow(deprecated)]
pub use sa_sso_util::SaSsoUtil;
