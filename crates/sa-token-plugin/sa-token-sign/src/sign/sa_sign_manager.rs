use crate::sign::{SaSignConfig, SaSignTemplate};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// Isolated owner of the default and named signature templates.
pub struct SaSignManager {
    default_template: Arc<SaSignTemplate>,
    named: RwLock<HashMap<String, Arc<SaSignConfig>>>,
}
impl SaSignManager {
    pub fn new(default_template: Arc<SaSignTemplate>) -> Self {
        Self {
            default_template,
            named: RwLock::new(HashMap::new()),
        }
    }
    pub fn default_template(&self) -> Arc<SaSignTemplate> {
        Arc::clone(&self.default_template)
    }
    pub fn register(
        &self,
        app_id: impl Into<String>,
        config: Arc<SaSignConfig>,
    ) -> Result<(), String> {
        self.named
            .write()
            .map_err(|error| error.to_string())?
            .insert(app_id.into(), config);
        Ok(())
    }
    pub fn config(&self, app_id: &str) -> Result<Option<Arc<SaSignConfig>>, String> {
        self.named
            .read()
            .map_err(|error| error.to_string())
            .map(|values| values.get(app_id).cloned())
    }
}
