/// OAuth2 grant type identifiers.
pub struct GrantType;

impl GrantType {
    pub const AUTHORIZATION_CODE: &'static str = "authorization_code";
    pub const REFRESH_TOKEN: &'static str = "refresh_token";
    pub const PASSWORD: &'static str = "password";
    pub const CLIENT_CREDENTIALS: &'static str = "client_credentials";
    pub const IMPLICIT: &'static str = "implicit";
}
