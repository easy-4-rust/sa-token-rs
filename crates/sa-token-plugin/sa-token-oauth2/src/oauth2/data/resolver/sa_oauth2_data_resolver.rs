use std::collections::BTreeMap;

use sa_token_core::exception::SaResult;
use serde_json::Value;

use crate::oauth2::data::model::request::{ClientIdAndSecretModel, RequestAuthModel};
use crate::oauth2::data::model::{AccessTokenModel, ClientTokenModel};

/// Framework-neutral OAuth2 request input.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SaOAuth2Request {
    pub params: BTreeMap<String, String>,
    pub headers: BTreeMap<String, String>,
}

impl SaOAuth2Request {
    pub fn param(&self, name: &str) -> Option<&str> {
        self.params
            .get(name)
            .map(String::as_str)
            .filter(|value| !value.is_empty())
    }

    pub fn header(&self, name: &str) -> Option<&str> {
        self.headers
            .get(name)
            .map(String::as_str)
            .filter(|value| !value.is_empty())
    }
}

pub type SaOAuth2Response = BTreeMap<String, Value>;

/// Request parsing and protocol response construction contract.
pub trait SaOAuth2DataResolver: Send + Sync {
    fn read_client_id_and_secret(
        &self,
        request: &SaOAuth2Request,
    ) -> SaResult<ClientIdAndSecretModel>;
    fn read_access_token(&self, request: &SaOAuth2Request) -> Option<String>;
    fn read_client_token(&self, request: &SaOAuth2Request) -> Option<String>;
    fn read_request_auth_model(
        &self,
        request: &SaOAuth2Request,
        login_id: Value,
    ) -> RequestAuthModel;
    fn build_access_token_return_value(&self, model: &AccessTokenModel) -> SaOAuth2Response;
    fn build_client_token_return_value(&self, model: &ClientTokenModel) -> SaOAuth2Response;

    fn build_refresh_token_return_value(&self, model: &AccessTokenModel) -> SaOAuth2Response {
        self.build_access_token_return_value(model)
    }

    fn build_revoke_token_return_value(&self) -> SaOAuth2Response {
        BTreeMap::from([
            ("code".into(), Value::from(200)),
            ("msg".into(), Value::from("ok")),
            ("data".into(), Value::Null),
        ])
    }
}
