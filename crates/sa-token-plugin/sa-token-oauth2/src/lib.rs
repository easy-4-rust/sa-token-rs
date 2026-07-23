//! Sa-Token OAuth2 authorization-server primitives.
//!
//! The crate exposes isolated sync-free runtime components under [`oauth2`]
//! and lifecycle integration under [`plugin`]. HTTP framework adapters build
//! on the processor API instead of relying on process-global request state.

pub mod oauth2;
pub mod plugin;
