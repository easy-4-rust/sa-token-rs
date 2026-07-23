use crate::sign::{SaSignErrorCode, SaSignException, SaSignManager, SaSignTemplate};
use std::sync::Arc;
/// Named signature-template selector.
pub struct SaSignMany {
    manager: Arc<SaSignManager>,
    factory: Arc<dyn Fn(Arc<crate::sign::SaSignConfig>) -> Arc<SaSignTemplate> + Send + Sync>,
}
impl SaSignMany {
    pub fn new(
        manager: Arc<SaSignManager>,
        factory: Arc<dyn Fn(Arc<crate::sign::SaSignConfig>) -> Arc<SaSignTemplate> + Send + Sync>,
    ) -> Self {
        Self { manager, factory }
    }
    pub fn get_sign_template(&self, app_id: &str) -> Result<Arc<SaSignTemplate>, SaSignException> {
        if app_id.is_empty() {
            return Ok(self.manager.default_template());
        }
        let config = self
            .manager
            .config(app_id)
            .map_err(|error| SaSignException::new(0, error))?
            .ok_or_else(|| {
                SaSignException::new(
                    SaSignErrorCode::CODE_12211,
                    format!("signature config not found: appid={app_id}"),
                )
            })?;
        Ok((self.factory)(config))
    }
}
