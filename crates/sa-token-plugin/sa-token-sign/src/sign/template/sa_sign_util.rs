use crate::sign::{SaSignException, SaSignTemplate};
use std::collections::HashMap;
use std::sync::Arc;
/// Explicit-runtime convenience facade for signature operations.
pub struct SaSignUtil {
    template: Arc<SaSignTemplate>,
}
impl SaSignUtil {
    pub fn new(template: Arc<SaSignTemplate>) -> Self {
        Self { template }
    }
    pub fn create_sign(&self, params: &HashMap<String, String>) -> Result<String, SaSignException> {
        self.template.create_sign(params)
    }
    pub fn check_param_map(&self, params: &HashMap<String, String>) -> Result<(), SaSignException> {
        self.template.check_param_map(params)
    }
}
