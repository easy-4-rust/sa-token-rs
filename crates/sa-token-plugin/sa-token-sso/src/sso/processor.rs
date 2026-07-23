//! Framework-neutral SSO request processors.

pub mod sa_sso_client_processor;
pub mod sa_sso_processor_helper;
pub mod sa_sso_server_processor;

pub use sa_sso_client_processor::{SaSsoClientProcessor, SaSsoClientSession};
pub use sa_sso_processor_helper::{SaSsoProcessorHelper, SaSsoProcessorResult, SaSsoRequest};
pub use sa_sso_server_processor::{SaSsoServerProcessor, SaSsoServerSession};
