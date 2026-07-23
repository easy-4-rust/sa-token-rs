use sa_token_core::stp::AsyncStpUtil;
use salvo::prelude::Request;

/// Extracts the Sa-Token credential from header, cookie, query, or Bearer authorization.
pub fn extract_token(request: &Request, util: &AsyncStpUtil) -> Option<String> {
    let token_name = &util.logic().runtime().config().token_name;
    request
        .header::<String>(token_name)
        .or_else(|| request.cookie(token_name).map(|cookie| cookie.value().to_string()))
        .or_else(|| {
            request
                .header::<String>("authorization")
                .and_then(|value| value.strip_prefix("Bearer ").map(str::to_string))
        })
        .or_else(|| request.query::<String>(token_name))
}
