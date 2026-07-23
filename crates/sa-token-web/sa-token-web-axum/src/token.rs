//! Token extraction helpers mapped from servlet/reactor operate utilities.

/// Extracts a token from normalized header/cookie tuples.
pub fn extract_token_from_headers(
    token_name: &str,
    headers: &[(String, String)],
    cookies: &[(String, String)],
) -> Option<String> {
    let token_name_lower = token_name.to_ascii_lowercase();
    headers
        .iter()
        .find(|(name, _)| name.to_ascii_lowercase() == token_name_lower)
        .map(|(_, value)| value.clone())
        .or_else(|| {
            cookies
                .iter()
                .find(|(name, _)| name == token_name)
                .map(|(_, value)| value.clone())
        })
        .or_else(|| {
            headers
                .iter()
                .find(|(name, _)| name.to_ascii_lowercase() == "authorization")
                .and_then(|(_, value)| value.strip_prefix("Bearer ").map(str::to_string))
        })
}

/// Extracts a token from Axum request parts using the configured token name.
pub fn extract_token_from_request_parts(
    token_name: &str,
    headers: &[(String, String)],
    cookies: &[(String, String)],
) -> Option<String> {
    extract_token_from_headers(token_name, headers, cookies)
}
