//! OAuth2 protocol models.

pub mod access_token_model;
pub mod client_token_model;
pub mod code_model;
pub mod loader;
pub mod oidc;
pub mod refresh_token_model;
pub mod request;

pub use access_token_model::AccessTokenModel;
pub use client_token_model::ClientTokenModel;
pub use code_model::CodeModel;
pub use refresh_token_model::RefreshTokenModel;

use std::time::{SystemTime, UNIX_EPOCH};

pub(crate) fn now_millis() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |duration| {
            i64::try_from(duration.as_millis()).unwrap_or(i64::MAX)
        })
}

pub(crate) fn expires_in_at(expires_time: i64, now_millis: i64) -> i64 {
    let seconds = (expires_time - now_millis) / 1_000;
    if seconds < 1 { -2 } else { seconds }
}
