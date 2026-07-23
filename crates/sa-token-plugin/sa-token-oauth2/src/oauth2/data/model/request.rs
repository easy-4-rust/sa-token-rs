//! Models parsed from OAuth2 requests.

pub mod client_id_and_secret_model;
pub mod request_auth_model;

pub use client_id_and_secret_model::ClientIdAndSecretModel;
pub use request_auth_model::RequestAuthModel;
