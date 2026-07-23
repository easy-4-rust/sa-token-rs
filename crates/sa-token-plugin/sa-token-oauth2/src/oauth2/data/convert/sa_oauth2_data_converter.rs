use serde_json::Value;

use crate::oauth2::data::model::loader::SaClientModel;
use crate::oauth2::data::model::request::RequestAuthModel;
use crate::oauth2::data::model::{
    AccessTokenModel, ClientTokenModel, CodeModel, RefreshTokenModel,
};

/// Token value generation strategy injected into the default converter.
pub trait SaOAuth2TokenGenerator: Send + Sync {
    type Error;

    fn create_code(
        &self,
        client_id: &str,
        login_id: &Value,
        scopes: &[String],
    ) -> Result<String, Self::Error>;
    fn create_access_token(
        &self,
        client_id: &str,
        login_id: &Value,
        scopes: &[String],
    ) -> Result<String, Self::Error>;
    fn create_refresh_token(
        &self,
        client_id: &str,
        login_id: &Value,
        scopes: &[String],
    ) -> Result<String, Self::Error>;
    fn create_client_token(
        &self,
        client_id: &str,
        scopes: &[String],
    ) -> Result<String, Self::Error>;
}

/// Java-compatible OAuth2 model conversion contract.
pub trait SaOAuth2DataConverter {
    type Error;

    fn convert_scope_string_to_list(&self, value: &str) -> Vec<String>;
    fn convert_scope_list_to_string(&self, value: &[String]) -> String;
    fn convert_redirect_uri_string_to_list(&self, value: &str) -> Vec<String>;
    fn convert_request_auth_to_code(
        &self,
        request: &RequestAuthModel,
    ) -> Result<CodeModel, Self::Error>;
    fn convert_request_auth_to_access_token(
        &self,
        request: &RequestAuthModel,
        timeout: i64,
    ) -> Result<AccessTokenModel, Self::Error>;
    fn convert_code_to_access_token(
        &self,
        code: &CodeModel,
        timeout: i64,
    ) -> Result<AccessTokenModel, Self::Error>;
    fn convert_access_token_to_refresh_token(
        &self,
        access_token: &AccessTokenModel,
        timeout: i64,
    ) -> Result<RefreshTokenModel, Self::Error>;
    fn convert_refresh_token_to_access_token(
        &self,
        refresh_token: &RefreshTokenModel,
        timeout: i64,
    ) -> Result<AccessTokenModel, Self::Error>;
    fn convert_refresh_token_to_refresh_token(
        &self,
        refresh_token: &RefreshTokenModel,
        timeout: i64,
    ) -> Result<RefreshTokenModel, Self::Error>;
    fn convert_sa_client_to_client_token(
        &self,
        client: &SaClientModel,
        scopes: &[String],
    ) -> Result<ClientTokenModel, Self::Error>;
}
