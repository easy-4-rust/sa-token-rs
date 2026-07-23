use std::collections::BTreeMap;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use serde_json::Value;

use crate::oauth2::config::SaOAuth2ServerConfig;
use crate::oauth2::consts::SaOAuth2ExtraField;
use crate::oauth2::data::model::oidc::IdTokenModel;
use crate::oauth2::data::model::{AccessTokenModel, ClientTokenModel};
use crate::oauth2::exception::SaOAuth2Exception;
use crate::oauth2::scope::CommonScope;

use super::SaOAuth2ScopeHandlerInterface;

/// Request and session values required to create an OpenID Connect ID token.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OidcScopeContext {
    pub issuer: String,
    pub audience: String,
    pub auth_time: i64,
    pub nonce: String,
}

/// Supplies framework-specific request and login-session values without global state.
pub trait OidcScopeContextProvider: Send + Sync {
    /// Resolves the context for an access-token model.
    ///
    /// # Errors
    ///
    /// Returns a protocol error when request, client, session, or nonce data is invalid.
    fn context(
        &self,
        access_token: &AccessTokenModel,
    ) -> Result<OidcScopeContext, SaOAuth2Exception>;
}

impl<F> OidcScopeContextProvider for F
where
    F: Fn(&AccessTokenModel) -> Result<OidcScopeContext, SaOAuth2Exception> + Send + Sync,
{
    fn context(
        &self,
        access_token: &AccessTokenModel,
    ) -> Result<OidcScopeContext, SaOAuth2Exception> {
        self(access_token)
    }
}

/// Signs an ID-token model using the application's JWT implementation.
pub trait SaOAuth2IdTokenGenerator: Send + Sync {
    /// Produces the compact JWT value.
    ///
    /// # Errors
    ///
    /// Returns a protocol error when signing configuration or claims are invalid.
    fn generate_id_token(&self, model: &IdTokenModel) -> Result<String, SaOAuth2Exception>;
}

/// Adds a signed `id_token` to access-token extra data.
pub struct OidcScopeHandler {
    config: Arc<SaOAuth2ServerConfig>,
    context_provider: Arc<dyn OidcScopeContextProvider>,
    generator: Arc<dyn SaOAuth2IdTokenGenerator>,
}

impl OidcScopeHandler {
    pub fn new(
        config: Arc<SaOAuth2ServerConfig>,
        context_provider: Arc<dyn OidcScopeContextProvider>,
        generator: Arc<dyn SaOAuth2IdTokenGenerator>,
    ) -> Self {
        Self {
            config,
            context_provider,
            generator,
        }
    }

    fn now_seconds() -> i64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |duration| match i64::try_from(duration.as_secs()) {
                Ok(seconds) => seconds,
                Err(_) => i64::MAX,
            })
    }

    pub fn build_id_token(
        &self,
        access_token: &AccessTokenModel,
    ) -> Result<IdTokenModel, SaOAuth2Exception> {
        let context = self.context_provider.context(access_token)?;
        let iat = Self::now_seconds();
        Ok(IdTokenModel {
            iss: Some(context.issuer),
            sub: access_token.login_id.clone(),
            aud: Some(context.audience.clone()),
            exp: iat.saturating_add(self.config.oidc.id_token_timeout),
            iat,
            auth_time: context.auth_time,
            nonce: Some(context.nonce),
            acr: None,
            amr: None,
            azp: Some(context.audience),
            extra_data: Some(BTreeMap::new()),
        })
    }
}

impl SaOAuth2ScopeHandlerInterface for OidcScopeHandler {
    fn handler_scope(&self) -> &str {
        CommonScope::OIDC
    }

    fn work_access_token(
        &self,
        access_token: &mut AccessTokenModel,
    ) -> Result<(), SaOAuth2Exception> {
        let id_token = self.build_id_token(access_token)?;
        let jwt = self.generator.generate_id_token(&id_token)?;
        access_token
            .extra_data
            .get_or_insert_default()
            .insert(SaOAuth2ExtraField::ID_TOKEN.into(), Value::String(jwt));
        Ok(())
    }

    fn work_client_token(&self, _: &mut ClientTokenModel) -> Result<(), SaOAuth2Exception> {
        Ok(())
    }

    fn refresh_access_token_is_work(&self) -> bool {
        true
    }
}
