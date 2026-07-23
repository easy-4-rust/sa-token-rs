use crate::sign::{SaCheckSign, SaSignException, SaSignMany};
use std::collections::HashMap;
use std::sync::Arc;
/// Annotation handler over an explicit named-template selector.
pub struct SaCheckSignHandler {
    sign_many: Arc<SaSignMany>,
}
impl SaCheckSignHandler {
    pub fn new(sign_many: Arc<SaSignMany>) -> Self {
        Self { sign_many }
    }
    pub fn check(
        &self,
        metadata: &SaCheckSign,
        request_params: &HashMap<String, String>,
    ) -> Result<(), SaSignException> {
        let app_id = if metadata.app_id.starts_with("#{") && metadata.app_id.ends_with('}') {
            request_params
                .get(&metadata.app_id[2..metadata.app_id.len() - 1])
                .map(String::as_str)
                .unwrap_or("")
        } else {
            metadata.app_id.as_str()
        };
        let template = self.sign_many.get_sign_template(app_id)?;
        if metadata.verify_params.is_empty() {
            return template.check_param_map(request_params);
        }
        let mut selected = HashMap::new();
        for required in [
            SaSignTemplateNames::TIMESTAMP,
            SaSignTemplateNames::NONCE,
            SaSignTemplateNames::SIGN,
        ] {
            if let Some(value) = request_params.get(required) {
                selected.insert(required.to_string(), value.clone());
            }
        }
        for name in &metadata.verify_params {
            if let Some(value) = request_params.get(name) {
                selected.insert(name.clone(), value.clone());
            }
        }
        template.check_param_map(&selected)
    }
}
struct SaSignTemplateNames;
impl SaSignTemplateNames {
    const TIMESTAMP: &'static str = "timestamp";
    const NONCE: &'static str = "nonce";
    const SIGN: &'static str = "sign";
}
