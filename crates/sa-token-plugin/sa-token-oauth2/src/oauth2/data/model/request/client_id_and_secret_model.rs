use std::fmt;

use serde::{Deserialize, Serialize};

/// Client credentials parsed from an OAuth2 request.
#[derive(Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct ClientIdAndSecretModel {
    pub client_id: Option<String>,
    pub client_secret: Option<String>,
}

impl ClientIdAndSecretModel {
    pub fn new(client_id: impl Into<String>, client_secret: impl Into<String>) -> Self {
        Self {
            client_id: Some(client_id.into()),
            client_secret: Some(client_secret.into()),
        }
    }
}

impl fmt::Debug for ClientIdAndSecretModel {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ClientIdAndSecretModel")
            .field("client_id", &self.client_id)
            .field("client_secret", &self.client_secret.as_ref().map(|_| "***"))
            .finish()
    }
}
