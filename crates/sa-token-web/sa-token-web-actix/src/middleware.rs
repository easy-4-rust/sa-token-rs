use actix_web::body::MessageBody;
use actix_web::dev::{ServiceRequest, ServiceResponse};
use actix_web::error::{ErrorInternalServerError, ErrorUnauthorized};
use actix_web::middleware::Next;
use actix_web::{Error, HttpMessage, web};
use sa_token_core::stp::AsyncStpUtil;

use crate::identity::LoginIdentity;
use crate::token::extract_token;

/// Middleware for routes that require authentication.
pub async fn require_login<B: MessageBody + 'static>(
    request: ServiceRequest,
    next: Next<B>,
) -> Result<ServiceResponse<B>, Error> {
    let util = request
        .app_data::<web::Data<AsyncStpUtil>>()
        .cloned()
        .ok_or_else(|| ErrorInternalServerError("AsyncStpUtil is not configured in Actix app data"))?;
    let token = extract_token(request.request()).ok_or_else(|| ErrorUnauthorized("authentication token is missing"))?;
    let login_id = util
        .get_login_id_by_token(&token)
        .await
        .map_err(ErrorInternalServerError)?
        .ok_or_else(|| ErrorUnauthorized("authentication token is invalid"))?;
    request.extensions_mut().insert(LoginIdentity { login_id, token });
    next.call(request).await
}
