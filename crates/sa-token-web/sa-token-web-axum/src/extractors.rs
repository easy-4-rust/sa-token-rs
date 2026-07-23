//! Login id extractors for Axum handlers.

use axum::http::StatusCode;
use sa_token_core::stp::stp_util::StpUtil;

/// 当前登录 ID Extractor
pub struct CurrentLoginId(pub String);

impl<S: Send + Sync> axum::extract::FromRequestParts<S> for CurrentLoginId {
    type Rejection = (StatusCode, String);

    async fn from_request_parts(
        _parts: &mut axum::http::request::Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        match StpUtil::get_login_id() {
            Ok(id) => Ok(CurrentLoginId(id)),
            Err(e) => Err((StatusCode::UNAUTHORIZED, e.to_string())),
        }
    }
}

/// 可选登录 ID Extractor
pub struct OptionalLoginId(pub Option<String>);

impl<S: Send + Sync> axum::extract::FromRequestParts<S> for OptionalLoginId {
    type Rejection = (StatusCode, String);

    async fn from_request_parts(
        _parts: &mut axum::http::request::Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        StpUtil::get_login_id_default_null()
            .map(OptionalLoginId)
            .map_err(|error| (StatusCode::INTERNAL_SERVER_ERROR, error.to_string()))
    }
}
