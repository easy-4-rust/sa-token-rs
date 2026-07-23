use std::future::{Ready, ready};

use actix_web::dev::Payload;
use actix_web::error::{ErrorInternalServerError, ErrorUnauthorized};
use actix_web::{Error, FromRequest, HttpMessage, HttpRequest, web};
use futures_util::future::LocalBoxFuture;
use sa_token_core::stp::AsyncStpUtil;

use crate::identity::LoginIdentity;
use crate::token::extract_token;

/// Actix extractor requiring an authenticated login.
pub struct RequireLogin(pub LoginIdentity);

impl FromRequest for RequireLogin {
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self, Self::Error>>;

    fn from_request(request: &HttpRequest, _payload: &mut Payload) -> Self::Future {
        if let Some(identity) = request.extensions().get::<LoginIdentity>().cloned() {
            return Box::pin(async move { Ok(Self(identity)) });
        }
        let util = request.app_data::<web::Data<AsyncStpUtil>>().cloned();
        let token = extract_token(request);
        Box::pin(async move {
            let util = util.ok_or_else(|| ErrorInternalServerError("AsyncStpUtil is not configured in Actix app data"))?;
            let token = token.ok_or_else(|| ErrorUnauthorized("authentication token is missing"))?;
            let login_id = util
                .get_login_id_by_token(&token)
                .await
                .map_err(ErrorInternalServerError)?
                .ok_or_else(|| ErrorUnauthorized("authentication token is invalid"))?;
            Ok(Self(LoginIdentity { login_id, token }))
        })
    }
}

/// Optional extractor that never rejects missing or invalid credentials.
pub struct OptionalLogin(pub Option<LoginIdentity>);

impl FromRequest for OptionalLogin {
    type Error = Error;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(request: &HttpRequest, _payload: &mut Payload) -> Self::Future {
        ready(Ok(Self(request.extensions().get::<LoginIdentity>().cloned())))
    }
}
