//! Config injection entry point (Java `SaQuickInject`).

use crate::quick::config::sa_quick_config::SaQuickConfig;
use crate::quick::sa_quick_manager::SaQuickManager;

/// Injects quick-login configuration into the global manager.
pub struct SaQuickInject;

impl SaQuickInject {
    /// Applies optional configuration, mirroring Spring `@Autowired` injection.
    pub fn inject(config: Option<SaQuickConfig>) {
        if let Some(cfg) = config {
            SaQuickManager::set_config(cfg);
        }
    }
}
