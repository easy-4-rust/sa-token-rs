use crate::sso::name::ParamName;
use crate::sso::util::SaSsoConsts;
use serde_json::Value;
use std::collections::HashMap;

/// Framework-neutral request view consumed by SSO processors.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SaSsoRequest {
    pub path: String,
    pub params: HashMap<String, String>,
}

impl SaSsoRequest {
    /// Creates a request for a path.
    pub fn new(path: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            params: HashMap::new(),
        }
    }

    /// Adds a query/form parameter.
    pub fn with_param(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.params.insert(key.into(), value.into());
        self
    }

    /// Returns a parameter value.
    pub fn param(&self, name: &str) -> Option<&str> {
        self.params.get(name).map(String::as_str)
    }
}

/// Portable result that each Web adapter converts into its native response.
#[derive(Debug, Clone, PartialEq)]
pub enum SaSsoProcessorResult {
    NotHandled,
    Json(Value),
    Redirect(String),
    Html(String),
}

/// Shared processor response rules.
pub struct SaSsoProcessorHelper;

impl SaSsoProcessorHelper {
    /// Produces the Java-compatible logout response.
    pub fn sso_logout_back(request: &SaSsoRequest, param_name: &ParamName) -> SaSsoProcessorResult {
        match request.param(&param_name.back).filter(|value| !value.is_empty()) {
            Some(SaSsoConsts::SELF) => SaSsoProcessorResult::Html(
                "<script>if(document.referrer != location.href){ location.replace(document.referrer || '/'); }</script>".into(),
            ),
            Some(back) => SaSsoProcessorResult::Redirect(back.to_owned()),
            None => SaSsoProcessorResult::Json(
                serde_json::json!({"code": 200, "msg": "单点注销成功"}),
            ),
        }
    }
}
