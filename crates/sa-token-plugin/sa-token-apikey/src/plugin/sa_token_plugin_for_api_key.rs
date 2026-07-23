//! API Key plugin lifecycle integration.

use std::any::Any;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use sa_token_core::plugin::sa_token_plugin::SaTokenPlugin;

use crate::apikey::annotation::SaCheckApiKeyHandler;

/// Lifecycle marker for API Key annotation integration.
pub struct SaTokenPluginForApiKey {
    installed: AtomicBool,
    handler: Option<Arc<SaCheckApiKeyHandler>>,
}

impl Default for SaTokenPluginForApiKey {
    fn default() -> Self {
        Self {
            installed: AtomicBool::new(false),
            handler: None,
        }
    }
}

impl SaTokenPluginForApiKey {
    /// Creates a plugin wired to an isolated annotation handler.
    pub fn with_handler(handler: Arc<SaCheckApiKeyHandler>) -> Self {
        Self {
            installed: AtomicBool::new(false),
            handler: Some(handler),
        }
    }

    /// Returns whether installation has run.
    pub fn is_installed(&self) -> bool {
        self.installed.load(Ordering::Acquire)
    }

    /// Returns the registered handler only while the plugin is installed.
    pub fn handler(&self) -> Option<&Arc<SaCheckApiKeyHandler>> {
        self.is_installed()
            .then_some(self.handler.as_ref())
            .flatten()
    }
}

impl SaTokenPlugin for SaTokenPluginForApiKey {
    fn install(&self) {
        self.installed.store(true, Ordering::Release);
    }

    fn destroy(&self) {
        self.installed.store(false, Ordering::Release);
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
