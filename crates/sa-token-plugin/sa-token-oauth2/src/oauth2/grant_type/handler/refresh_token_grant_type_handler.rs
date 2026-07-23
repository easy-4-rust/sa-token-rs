use std::sync::Arc;

use async_trait::async_trait;
use sa_token_core::exception::{SaResult, SaTokenException};

use crate::oauth2::consts::{GrantType, SaOAuth2Param};
use crate::oauth2::dao::SaOAuth2Dao;
use crate::oauth2::data::generate::SaOAuth2DataGenerate;
use crate::oauth2::data::model::AccessTokenModel;
use crate::oauth2::data::resolver::SaOAuth2Request;

use super::SaOAuth2GrantTypeHandlerInterface;

/// Refresh-token grant handler backed by the async OAuth2 DAO.
pub struct RefreshTokenGrantTypeHandler {
    dao: Arc<SaOAuth2Dao>,
    generator: Arc<dyn SaOAuth2DataGenerate>,
}

impl RefreshTokenGrantTypeHandler {
    pub fn new(dao: Arc<SaOAuth2Dao>, generator: Arc<dyn SaOAuth2DataGenerate>) -> Self {
        Self { dao, generator }
    }
}

#[async_trait]
impl SaOAuth2GrantTypeHandlerInterface for RefreshTokenGrantTypeHandler {
    fn handler_grant_type(&self) -> &str {
        GrantType::REFRESH_TOKEN
    }

    async fn get_access_token(
        &self,
        request: &SaOAuth2Request,
        client_id: &str,
        _: &[String],
    ) -> SaResult<AccessTokenModel> {
        let refresh_token = request.param(SaOAuth2Param::REFRESH_TOKEN).ok_or_else(|| {
            SaTokenException::with_code(
                30191,
                format!("缺少参数: {}", SaOAuth2Param::REFRESH_TOKEN),
            )
        })?;
        let model = self
            .dao
            .get_refresh_token(refresh_token)
            .await?
            .ok_or_else(|| {
                SaTokenException::with_code(30111, format!("无效 refresh_token: {refresh_token}"))
            })?;
        if model.client_id.as_deref() != Some(client_id) {
            return Err(SaTokenException::with_code(
                30122,
                format!("无效 client_id: {client_id}"),
            ));
        }
        self.generator.refresh_access_token(refresh_token).await
    }
}
