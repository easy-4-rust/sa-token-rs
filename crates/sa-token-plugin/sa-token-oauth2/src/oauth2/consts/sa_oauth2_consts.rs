/// Top-level OAuth2 constants.
pub struct SaOAuth2Consts;

impl SaOAuth2Consts {
    pub const OPENID_DEFAULT_DIGEST_PREFIX: &'static str = "openid_default_digest_prefix";
    pub const UNIONID_DEFAULT_DIGEST_PREFIX: &'static str = "unionid_default_digest_prefix";
    pub const OK: &'static str = "ok";
    pub const NOT_HANDLE: &'static str = "{\"msg\": \"not handle\"}";
    pub const FINALLY_WORK_SCOPE: &'static str = "_FINALLY_WORK_SCOPE";
}

/// OAuth2 endpoint paths.
pub struct SaOAuth2Api;

impl SaOAuth2Api {
    pub const AUTHORIZE: &'static str = "/oauth2/authorize";
    pub const TOKEN: &'static str = "/oauth2/token";
    pub const REFRESH: &'static str = "/oauth2/refresh";
    pub const REVOKE: &'static str = "/oauth2/revoke";
    pub const CLIENT_TOKEN: &'static str = "/oauth2/client_token";
    pub const DO_LOGIN: &'static str = "/oauth2/doLogin";
    pub const DO_CONFIRM: &'static str = "/oauth2/doConfirm";
}

/// OAuth2 request parameter names.
pub struct SaOAuth2Param;

impl SaOAuth2Param {
    pub const RESPONSE_TYPE: &'static str = "response_type";
    pub const CLIENT_ID: &'static str = "client_id";
    pub const CLIENT_SECRET: &'static str = "client_secret";
    pub const REDIRECT_URI: &'static str = "redirect_uri";
    pub const SCOPE: &'static str = "scope";
    pub const STATE: &'static str = "state";
    pub const CODE: &'static str = "code";
    pub const TOKEN: &'static str = "token";
    pub const ACCESS_TOKEN: &'static str = "access_token";
    pub const REFRESH_TOKEN: &'static str = "refresh_token";
    pub const CLIENT_TOKEN: &'static str = "client_token";
    pub const GRANT_TYPE: &'static str = "grant_type";
    pub const USERNAME: &'static str = "username";
    pub const PASSWORD: &'static str = "password";
    pub const NAME: &'static str = "name";
    pub const PWD: &'static str = "pwd";
    pub const BUILD_REDIRECT_URI: &'static str = "build_redirect_uri";
    pub const AUTHORIZATION: &'static str = "Authorization";
    pub const NONCE: &'static str = "nonce";
}

/// OAuth2 response type identifiers.
pub struct SaOAuth2ResponseType;

impl SaOAuth2ResponseType {
    pub const CODE: &'static str = "code";
    pub const TOKEN: &'static str = "token";
}

/// OAuth2 HTTP authorization schemes.
pub struct SaOAuth2TokenType;

impl SaOAuth2TokenType {
    pub const BASIC: &'static str = "basic";
    pub const DIGEST: &'static str = "digest";
    pub const BEARER: &'static str = "bearer";
    pub const BASIC_TITLE: &'static str = "Basic";
    pub const DIGEST_TITLE: &'static str = "Digest";
    pub const BEARER_TITLE: &'static str = "Bearer";
}

/// OAuth2/OIDC extension response fields.
pub struct SaOAuth2ExtraField;

impl SaOAuth2ExtraField {
    pub const UNION_ID: &'static str = "unionid";
    pub const OPEN_ID: &'static str = "openid";
    pub const USER_ID: &'static str = "userid";
    pub const ID_TOKEN: &'static str = "id_token";
}
