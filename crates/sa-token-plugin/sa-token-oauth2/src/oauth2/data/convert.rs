//! Conversions between OAuth2 request and persisted models.

pub mod sa_oauth2_data_converter;
pub mod sa_oauth2_data_converter_default_impl;

pub use sa_oauth2_data_converter::{SaOAuth2DataConverter, SaOAuth2TokenGenerator};
pub use sa_oauth2_data_converter_default_impl::SaOAuth2DataConverterDefaultImpl;
