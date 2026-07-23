/// Constants from Java `SaSsoConsts`.
pub struct SaSsoConsts;
impl SaSsoConsts {
    pub const SLO_CALLBACK_SET_KEY: &'static str = "SLO_CALLBACK_SET_KEY_";
    pub const SSO_CLIENT_MODEL_LIST_KEY: &'static str = "SSO_CLIENT_MODEL_LIST_KEY_";
    pub const OK: &'static str = "ok";
    pub const SELF: &'static str = "self";
    pub const MODE_SIMPLE: &'static str = "simple";
    pub const MODE_TICKET: &'static str = "ticket";
    pub const NOT_HANDLE: &'static str = "{\"msg\": \"not handle\"}";
    pub const CLIENT_WILDCARD: &'static str = "*";
    pub const CLIENT_ANON: &'static str = "anon";
    pub const SSO_MODE_1: i32 = 1;
    pub const SSO_MODE_2: i32 = 2;
    pub const SSO_MODE_3: i32 = 3;
    pub const MESSAGE_CHECK_TICKET: &'static str = "checkTicket";
    pub const MESSAGE_SIGNOUT: &'static str = "signout";
    pub const MESSAGE_LOGOUT_CALL: &'static str = "logoutCall";
}
