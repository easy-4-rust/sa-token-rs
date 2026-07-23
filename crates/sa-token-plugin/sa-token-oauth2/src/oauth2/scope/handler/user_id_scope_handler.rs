use crate::oauth2::consts::SaOAuth2ExtraField;
use crate::oauth2::data::model::{AccessTokenModel, ClientTokenModel};
use crate::oauth2::exception::SaOAuth2Exception;
use crate::oauth2::scope::CommonScope;

use super::SaOAuth2ScopeHandlerInterface;

/// Copies the access-token login ID into the `userid` extra field.
pub struct UserIdScopeHandler;

impl SaOAuth2ScopeHandlerInterface for UserIdScopeHandler {
    fn handler_scope(&self) -> &str {
        CommonScope::USERID
    }

    fn work_access_token(
        &self,
        access_token: &mut AccessTokenModel,
    ) -> Result<(), SaOAuth2Exception> {
        access_token.extra_data.get_or_insert_default().insert(
            SaOAuth2ExtraField::USER_ID.into(),
            access_token.login_id.clone().unwrap_or_default(),
        );
        Ok(())
    }

    fn work_client_token(&self, _: &mut ClientTokenModel) -> Result<(), SaOAuth2Exception> {
        Ok(())
    }
}
