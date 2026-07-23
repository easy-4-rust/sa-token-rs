//! Actix Web integration for asynchronous Sa-Token runtimes.

mod extractors;
mod identity;
pub mod mapping;
mod middleware;
mod runtime_wiring;
mod token;

pub use extractors::{OptionalLogin, RequireLogin};
pub use identity::LoginIdentity;
pub use middleware::require_login;
pub use runtime_wiring::register_async_runtime;
pub use token::extract_token;
