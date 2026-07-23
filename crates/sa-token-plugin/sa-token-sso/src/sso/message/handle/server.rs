//! Server-side SSO message handlers.

pub mod sa_sso_message_check_ticket_handle;
pub mod sa_sso_message_signout_handle;

pub use sa_sso_message_check_ticket_handle::SaSsoMessageCheckTicketHandle;
pub use sa_sso_message_signout_handle::SaSsoMessageSignoutHandle;
