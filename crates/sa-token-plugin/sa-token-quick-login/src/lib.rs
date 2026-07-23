//! Quick login plugin (Java `sa-token-quick-login`).

pub mod quick;

pub use quick::{
    DoLoginHandleFunction, SaQuickConfig, SaQuickController, SaQuickInject, SaQuickManager,
    SaQuickRegister,
};
