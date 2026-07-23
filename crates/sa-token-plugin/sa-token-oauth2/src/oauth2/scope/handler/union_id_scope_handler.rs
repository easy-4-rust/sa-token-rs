use std::sync::Arc;

use serde_json::Value;

use crate::oauth2::consts::SaOAuth2ExtraField;
use crate::oauth2::data::loader::SaOAuth2DataLoader;
use crate::oauth2::data::model::{AccessTokenModel, ClientTokenModel};
use crate::oauth2::exception::SaOAuth2Exception;
use crate::oauth2::scope::CommonScope;

use super::SaOAuth2ScopeHandlerInterface;
use super::sa_oauth2_scope_handler_interface::login_id_string;

/// Adds the subject-grouped `unionid` field to access-token extra data.
pub struct UnionIdScopeHandler {
    loader: Arc<dyn SaOAuth2DataLoader>,
}

impl UnionIdScopeHandler {
    pub fn new(loader: Arc<dyn SaOAuth2DataLoader>) -> Self {
        Self { loader }
    }
}

impl SaOAuth2ScopeHandlerInterface for UnionIdScopeHandler {
    fn handler_scope(&self) -> &str {
        CommonScope::UNIONID
    }

    fn work_access_token(
        &self,
        access_token: &mut AccessTokenModel,
    ) -> Result<(), SaOAuth2Exception> {
        let client_id = access_token.client_id.as_deref().unwrap_or_default();
        let client = self
            .loader
            .get_client_model_not_null(client_id)
            .map_err(|error| SaOAuth2Exception::new(error.base.message, error.base.code))?;
        let unionid = self.loader.get_unionid(
            client.subject_id.as_deref().unwrap_or_default(),
            &login_id_string(access_token.login_id.as_ref()),
        );
        access_token
            .extra_data
            .get_or_insert_default()
            .insert(SaOAuth2ExtraField::UNION_ID.into(), Value::String(unionid));
        Ok(())
    }

    fn work_client_token(&self, _: &mut ClientTokenModel) -> Result<(), SaOAuth2Exception> {
        Ok(())
    }
}
