/// Configurable SSO request parameter names.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParamName {
    pub redirect: String,
    pub ticket: String,
    pub back: String,
    pub mode: String,
    pub login_id: String,
    pub client: String,
    pub token_name: String,
    pub token_value: String,
    pub device_id: String,
    pub secret_key: String,
    pub sso_logout_call: String,
    pub auto_logout: String,
    pub name: String,
    pub pwd: String,
    pub timestamp: String,
    pub nonce: String,
    pub sign: String,
    pub remain_session_timeout: String,
    pub remain_token_timeout: String,
    pub single_device_id_logout: String,
}
impl Default for ParamName {
    fn default() -> Self {
        Self {
            redirect: "redirect".into(),
            ticket: "ticket".into(),
            back: "back".into(),
            mode: "mode".into(),
            login_id: "loginId".into(),
            client: "client".into(),
            token_name: "tokenName".into(),
            token_value: "tokenValue".into(),
            device_id: "deviceId".into(),
            secret_key: "secretkey".into(),
            sso_logout_call: "ssoLogoutCall".into(),
            auto_logout: "autoLogout".into(),
            name: "name".into(),
            pwd: "pwd".into(),
            timestamp: "timestamp".into(),
            nonce: "nonce".into(),
            sign: "sign".into(),
            remain_session_timeout: "remainSessionTimeout".into(),
            remain_token_timeout: "remainTokenTimeout".into(),
            single_device_id_logout: "singleDeviceIdLogout".into(),
        }
    }
}
