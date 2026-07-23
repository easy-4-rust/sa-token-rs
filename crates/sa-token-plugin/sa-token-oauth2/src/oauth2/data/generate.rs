//! OAuth2 authorization artifact generation.

pub mod sa_oauth2_data_generate;
pub mod sa_oauth2_data_generate_default_impl;

pub use sa_oauth2_data_generate::{SaOAuth2DataGenerate, SaOAuth2GenerateHooks};
pub use sa_oauth2_data_generate_default_impl::SaOAuth2DataGenerateDefaultImpl;
