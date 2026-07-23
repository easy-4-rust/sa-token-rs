use async_trait::async_trait;
use sa_token_core::exception::SaResult;
use serde_json::Value;

use crate::oauth2::data::model::request::RequestAuthModel;
use crate::oauth2::data::model::{AccessTokenModel, ClientTokenModel, CodeModel};

/// Explicit authorization and scope hooks used by the generator.
pub trait SaOAuth2GenerateHooks: Send + Sync {
    fn user_authorize_client_check(&self, _: &Value, _: &str) -> SaResult<()> {
        Ok(())
    }

    fn work_access_token_by_scope(&self, _: &mut AccessTokenModel) -> SaResult<()> {
        Ok(())
    }

    fn refresh_access_token_work_by_scope(&self, _: &mut AccessTokenModel) -> SaResult<()> {
        Ok(())
    }

    fn work_client_token_by_scope(&self, _: &mut ClientTokenModel) -> SaResult<()> {
        Ok(())
    }
}

/// Async Rust counterpart of Java `SaOAuth2DataGenerate`.
#[async_trait]
pub trait SaOAuth2DataGenerate: Send + Sync {
    async fn generate_code(&self, request: &RequestAuthModel) -> SaResult<CodeModel>;
    async fn generate_access_token(&self, code: &str) -> SaResult<AccessTokenModel>;
    async fn refresh_access_token(&self, refresh_token: &str) -> SaResult<AccessTokenModel>;
    async fn generate_access_token_by_request(
        &self,
        request: &RequestAuthModel,
        create_refresh_token: bool,
    ) -> SaResult<AccessTokenModel>;
    async fn generate_client_token(
        &self,
        client_id: &str,
        scopes: &[String],
    ) -> SaResult<ClientTokenModel>;
    async fn build_redirect_uri(
        &self,
        redirect_uri: &str,
        code: &str,
        state: Option<&str>,
    ) -> SaResult<String>;
    async fn build_implicit_redirect_uri(
        &self,
        redirect_uri: &str,
        token: &str,
        state: Option<&str>,
    ) -> SaResult<String>;
    async fn check_state(&self, state: &str) -> SaResult<()>;
}
