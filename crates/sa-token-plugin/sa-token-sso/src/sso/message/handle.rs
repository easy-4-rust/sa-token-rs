//! SSO message handler contracts.

pub mod client;
pub mod sa_sso_message_handle;
pub mod sa_sso_message_simple_handle;
pub mod server;

pub use client::SaSsoMessageLogoutCallHandle;
pub use sa_sso_message_handle::SaSsoMessageHandle;
pub use sa_sso_message_simple_handle::SaSsoMessageSimpleHandle;
pub use server::{SaSsoMessageCheckTicketHandle, SaSsoMessageSignoutHandle};
