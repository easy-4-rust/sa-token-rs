//! Salvo integration for asynchronous Sa-Token runtimes.

mod login_id;
pub mod mapping;
mod reject;
mod require_login;
mod token;

pub use login_id::{LOGIN_ID_KEY, TOKEN_KEY, login_id};
pub use require_login::RequireLogin;
pub use token::extract_token;
