//! Global quick-login config holder (Java `SaQuickManager`).

use std::sync::{OnceLock, RwLock};

use super::config::sa_quick_config::SaQuickConfig;

static CONFIG: OnceLock<RwLock<SaQuickConfig>> = OnceLock::new();

fn config_lock() -> &'static RwLock<SaQuickConfig> {
    CONFIG.get_or_init(|| RwLock::new(SaQuickConfig::default()))
}

/// Holds the global [`SaQuickConfig`] reference.
pub struct SaQuickManager;

impl SaQuickManager {
    /// Sets global quick-login configuration.
    pub fn set_config(mut config: SaQuickConfig) {
        config.apply_auto_credentials();
        *config_lock()
            .write()
            .unwrap_or_else(|poisoned| poisoned.into_inner()) = config;
    }

    /// Returns global quick-login configuration.
    pub fn get_config() -> SaQuickConfig {
        config_lock()
            .read()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lazy_default_config() {
        let cfg = SaQuickManager::get_config();
        assert_eq!(cfg.name, "sa");
    }
}
