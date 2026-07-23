//! Framework-neutral OAuth2 request and response resolution.

pub mod sa_oauth2_data_resolver;
pub mod sa_oauth2_data_resolver_default_impl;

pub use sa_oauth2_data_resolver::{SaOAuth2DataResolver, SaOAuth2Request, SaOAuth2Response};
pub use sa_oauth2_data_resolver_default_impl::SaOAuth2DataResolverDefaultImpl;
