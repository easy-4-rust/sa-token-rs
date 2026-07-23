use actix_web::http::header::AUTHORIZATION;
use actix_web::{HttpRequest, web};
use sa_token_core::stp::AsyncStpUtil;

/// Extracts the Sa-Token credential from header, cookie, or Bearer authorization.
pub fn extract_token(request: &HttpRequest) -> Option<String> {
    let token_name = request
        .app_data::<web::Data<AsyncStpUtil>>()
        .map(|util| util.logic().runtime().config().token_name.as_str())
        .unwrap_or("satoken");
    request
        .headers()
        .get(token_name)
        .and_then(|value| value.to_str().ok())
        .map(str::to_string)
        .or_else(|| request.cookie(token_name).map(|cookie| cookie.value().to_string()))
        .or_else(|| {
            request
                .headers()
                .get(AUTHORIZATION)
                .and_then(|value| value.to_str().ok())
                .and_then(|value| value.strip_prefix("Bearer "))
                .map(str::to_string)
        })
}
