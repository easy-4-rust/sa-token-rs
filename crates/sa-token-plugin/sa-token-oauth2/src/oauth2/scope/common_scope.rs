/// Common OAuth2 and OpenID Connect scopes.
pub struct CommonScope;

impl CommonScope {
    pub const OPENID: &'static str = "openid";
    pub const UNIONID: &'static str = "unionid";
    pub const USERID: &'static str = "userid";
    pub const OIDC: &'static str = "oidc";
}
