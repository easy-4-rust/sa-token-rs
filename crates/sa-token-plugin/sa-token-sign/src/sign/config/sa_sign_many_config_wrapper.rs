use super::SaSignConfig;
use std::collections::HashMap;
/// Framework-binding wrapper for named signature configurations.
#[derive(Debug, Clone, Default)]
pub struct SaSignManyConfigWrapper {
    pub sign_many: HashMap<String, SaSignConfig>,
}
