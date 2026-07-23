use serde_json::Value;

/// Result returned by password grant authentication.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct PasswordAuthResult {
    pub login_id: Option<Value>,
}

impl PasswordAuthResult {
    pub fn new(login_id: Value) -> Self {
        Self {
            login_id: Some(login_id),
        }
    }

    pub fn set_login_id(&mut self, login_id: Value) -> &mut Self {
        self.login_id = Some(login_id);
        self
    }
}
