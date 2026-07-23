use std::sync::Arc;

use sa_token_core::exception::SaResult;

use crate::oauth2::data::model::loader::SaClientModel;
use crate::oauth2::data::model::{AccessTokenModel, ClientTokenModel};

use super::SaOAuth2Template;

/// Instance facade over [`SaOAuth2Template`] without process-wide mutable state.
#[derive(Clone)]
pub struct SaOAuth2Util {
    template: Arc<SaOAuth2Template>,
}

impl SaOAuth2Util {
    pub fn new(template: Arc<SaOAuth2Template>) -> Self {
        Self { template }
    }

    pub fn template(&self) -> &Arc<SaOAuth2Template> {
        &self.template
    }

    pub fn check_client_model(&self, client_id: &str) -> SaResult<SaClientModel> {
        self.template.check_client_model(client_id)
    }

    pub fn check_client_secret_and_scope(
        &self,
        client_id: &str,
        client_secret: &str,
        scopes: &[String],
    ) -> SaResult<SaClientModel> {
        self.template
            .check_client_secret_and_scope(client_id, client_secret, scopes)
    }

    pub fn is_contract_scope(&self, client_id: &str, scopes: &[String]) -> bool {
        self.template.is_contract_scope(client_id, scopes)
    }

    pub async fn check_access_token(&self, token: &str) -> SaResult<AccessTokenModel> {
        self.template.check_access_token(token).await
    }

    pub async fn check_access_token_scope(
        &self,
        token: &str,
        scopes: &[String],
    ) -> SaResult<AccessTokenModel> {
        self.template.check_access_token_scope(token, scopes).await
    }

    pub async fn check_client_token(&self, token: &str) -> SaResult<ClientTokenModel> {
        self.template.check_client_token(token).await
    }

    pub async fn check_client_token_scope(
        &self,
        token: &str,
        scopes: &[String],
    ) -> SaResult<ClientTokenModel> {
        self.template.check_client_token_scope(token, scopes).await
    }
}
