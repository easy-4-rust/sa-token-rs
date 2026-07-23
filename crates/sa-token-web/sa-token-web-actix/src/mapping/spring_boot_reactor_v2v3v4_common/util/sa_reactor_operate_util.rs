//! Web integration mapping for Java `SaReactorOperateUtil`.
//! Responsibility is implemented by the `actix` adapter instead of Spring/Servlet crates.
pub use crate::token::extract_token as read_token_from_request;
