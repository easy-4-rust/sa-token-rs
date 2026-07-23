//! SSO protocol messages and their handler registry.

pub mod handle;
pub mod sa_sso_message;
pub mod sa_sso_message_holder;

pub use handle::{
    SaSsoMessageCheckTicketHandle, SaSsoMessageHandle, SaSsoMessageLogoutCallHandle,
    SaSsoMessageSignoutHandle, SaSsoMessageSimpleHandle,
};
pub use sa_sso_message::SaSsoMessage;
pub use sa_sso_message_holder::SaSsoMessageHolder;
