//! JWT-backed temporary token plugin (Java `sa-token-temp-jwt`).

pub mod plugin;
pub mod temp;

pub use plugin::SaTokenPluginForTempForJwt;
pub use temp::jwt::{
    SaJwtUtil, SaTempJwtErrorCode, SaTempTemplateForJwt,
};
