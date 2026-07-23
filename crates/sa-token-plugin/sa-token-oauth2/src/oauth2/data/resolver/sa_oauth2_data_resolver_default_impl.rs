use std::collections::BTreeMap;

use sa_token_core::exception::{SaResult, SaTokenException};
use sa_token_core::secure::sa_base64_util::SaBase64Util;
use serde_json::Value;

use crate::oauth2::data::model::request::{ClientIdAndSecretModel, RequestAuthModel};
use crate::oauth2::data::model::{AccessTokenModel, ClientTokenModel};

use super::{SaOAuth2DataResolver, SaOAuth2Request, SaOAuth2Response};

/// Default request parser and token response builder.
pub struct SaOAuth2DataResolverDefaultImpl {
    hide_status_field: bool,
    mode4_return_access_token: bool,
}

impl SaOAuth2DataResolverDefaultImpl {
    pub fn new(hide_status_field: bool, mode4_return_access_token: bool) -> Self {
        Self {
            hide_status_field,
            mode4_return_access_token,
        }
    }

    fn scopes(value: Option<&str>) -> Vec<String> {
        value
            .unwrap_or_default()
            .replace("%20", ",")
            .replace([' ', '+'], ",")
            .split(',')
            .map(str::trim)
            .filter(|scope| !scope.is_empty())
            .map(str::to_owned)
            .collect()
    }

    fn bearer(request: &SaOAuth2Request, parameter: &str) -> Option<String> {
        request.param(parameter).map(str::to_owned).or_else(|| {
            request
                .header("Authorization")
                .and_then(|value| value.strip_prefix("Bearer "))
                .filter(|value| !value.is_empty())
                .map(str::to_owned)
        })
    }

    fn basic_credentials(request: &SaOAuth2Request) -> Option<(String, Option<String>)> {
        let encoded = request.header("Authorization")?.strip_prefix("Basic ")?;
        let decoded = SaBase64Util::decode(encoded).ok()?;
        let decoded = String::from_utf8(decoded).ok()?;
        let mut parts = decoded.splitn(2, ':');
        let client_id = parts.next()?.to_owned();
        let secret = parts
            .next()
            .filter(|value| !value.is_empty())
            .map(str::to_owned);
        Some((client_id, secret))
    }

    fn base_response(&self) -> SaOAuth2Response {
        if self.hide_status_field {
            BTreeMap::new()
        } else {
            BTreeMap::from([
                ("code".into(), Value::from(200)),
                ("msg".into(), Value::from("ok")),
                ("data".into(), Value::Null),
            ])
        }
    }
}

impl SaOAuth2DataResolver for SaOAuth2DataResolverDefaultImpl {
    fn read_client_id_and_secret(
        &self,
        request: &SaOAuth2Request,
    ) -> SaResult<ClientIdAndSecretModel> {
        let client_id = request.param("client_id");
        let client_secret = request.param("client_secret");
        if let (Some(client_id), Some(client_secret)) = (client_id, client_secret) {
            return Ok(ClientIdAndSecretModel::new(client_id, client_secret));
        }
        if let Some((client_id, client_secret)) = Self::basic_credentials(request) {
            return Ok(ClientIdAndSecretModel {
                client_id: Some(client_id),
                client_secret,
            });
        }
        if let Some(client_id) = client_id {
            return Ok(ClientIdAndSecretModel {
                client_id: Some(client_id.into()),
                client_secret: None,
            });
        }
        Err(SaTokenException::with_code(30191, "请提供 client 信息"))
    }

    fn read_access_token(&self, request: &SaOAuth2Request) -> Option<String> {
        Self::bearer(request, "access_token")
    }

    fn read_client_token(&self, request: &SaOAuth2Request) -> Option<String> {
        Self::bearer(request, "client_token")
    }

    fn read_request_auth_model(
        &self,
        request: &SaOAuth2Request,
        login_id: Value,
    ) -> RequestAuthModel {
        RequestAuthModel {
            client_id: request.param("client_id").map(str::to_owned),
            scopes: Some(Self::scopes(request.param("scope"))),
            login_id: Some(login_id),
            redirect_uri: request.param("redirect_uri").map(str::to_owned),
            response_type: request.param("response_type").map(str::to_owned),
            state: request.param("state").map(str::to_owned),
            nonce: request.param("nonce").map(str::to_owned),
        }
    }

    fn build_access_token_return_value(&self, model: &AccessTokenModel) -> SaOAuth2Response {
        let mut response = self.base_response();
        response.insert(
            "token_type".into(),
            model.token_type.clone().map_or(Value::Null, Value::from),
        );
        response.insert(
            "access_token".into(),
            model.access_token.clone().map_or(Value::Null, Value::from),
        );
        response.insert(
            "refresh_token".into(),
            model.refresh_token.clone().map_or(Value::Null, Value::from),
        );
        response.insert("expires_in".into(), Value::from(model.expires_in()));
        response.insert(
            "refresh_expires_in".into(),
            Value::from(model.refresh_expires_in()),
        );
        response.insert(
            "client_id".into(),
            model.client_id.clone().map_or(Value::Null, Value::from),
        );
        response.insert(
            "scope".into(),
            Value::from(model.scopes.as_deref().unwrap_or_default().join(",")),
        );
        if let Some(extra) = &model.extra_data {
            response.extend(extra.clone());
        }
        response
    }

    fn build_client_token_return_value(&self, model: &ClientTokenModel) -> SaOAuth2Response {
        let mut response = self.base_response();
        let token = model.client_token.clone().map_or(Value::Null, Value::from);
        response.insert(
            "token_type".into(),
            model.token_type.clone().map_or(Value::Null, Value::from),
        );
        response.insert("client_token".into(), token.clone());
        if self.mode4_return_access_token {
            response.insert("access_token".into(), token);
        }
        response.insert("expires_in".into(), Value::from(model.expires_in()));
        response.insert(
            "client_id".into(),
            model.client_id.clone().map_or(Value::Null, Value::from),
        );
        response.insert(
            "scope".into(),
            Value::from(model.scopes.as_deref().unwrap_or_default().join(",")),
        );
        if let Some(extra) = &model.extra_data {
            response.extend(extra.clone());
        }
        response
    }
}
