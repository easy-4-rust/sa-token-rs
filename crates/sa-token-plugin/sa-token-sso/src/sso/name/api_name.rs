/// Configurable SSO endpoint names.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApiName {
    pub sso_auth: String,
    pub sso_do_login: String,
    pub sso_check_ticket: String,
    pub sso_push_s: String,
    pub sso_userinfo: String,
    pub sso_signout: String,
    pub sso_login: String,
    pub sso_logout: String,
    pub sso_is_login: String,
    pub sso_logout_call: String,
    pub sso_push_c: String,
}
impl Default for ApiName {
    fn default() -> Self {
        Self {
            sso_auth: "/sso/auth".into(),
            sso_do_login: "/sso/doLogin".into(),
            sso_check_ticket: "/sso/checkTicket".into(),
            sso_push_s: "/sso/pushS".into(),
            sso_userinfo: "/sso/userinfo".into(),
            sso_signout: "/sso/signout".into(),
            sso_login: "/sso/login".into(),
            sso_logout: "/sso/logout".into(),
            sso_is_login: "/sso/isLogin".into(),
            sso_logout_call: "/sso/logoutCall".into(),
            sso_push_c: "/sso/pushC".into(),
        }
    }
}
impl ApiName {
    pub fn add_prefix(mut self, prefix: &str) -> Self {
        self.map_paths(|path| format!("{prefix}{path}"));
        self
    }
    pub fn replace_prefix(mut self, prefix: &str) -> Self {
        self.map_paths(|path| path.replacen("/sso", prefix, 1));
        self
    }
    fn map_paths(&mut self, map: impl Fn(&str) -> String) {
        self.sso_auth = map(&self.sso_auth);
        self.sso_do_login = map(&self.sso_do_login);
        self.sso_check_ticket = map(&self.sso_check_ticket);
        self.sso_push_s = map(&self.sso_push_s);
        self.sso_userinfo = map(&self.sso_userinfo);
        self.sso_signout = map(&self.sso_signout);
        self.sso_login = map(&self.sso_login);
        self.sso_logout = map(&self.sso_logout);
        self.sso_is_login = map(&self.sso_is_login);
        self.sso_logout_call = map(&self.sso_logout_call);
        self.sso_push_c = map(&self.sso_push_c);
    }
}
