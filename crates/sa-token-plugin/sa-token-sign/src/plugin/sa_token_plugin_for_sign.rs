use crate::sign::SaCheckSignHandler;
use sa_token_core::plugin::sa_token_plugin::SaTokenPlugin;
use std::any::Any;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
/// Signature annotation plugin lifecycle.
pub struct SaTokenPluginForSign {
    installed: AtomicBool,
    handler: Arc<SaCheckSignHandler>,
}
impl SaTokenPluginForSign {
    pub fn new(handler: Arc<SaCheckSignHandler>) -> Self {
        Self {
            installed: AtomicBool::new(false),
            handler,
        }
    }
    pub fn handler(&self) -> Option<&Arc<SaCheckSignHandler>> {
        self.installed
            .load(Ordering::Acquire)
            .then_some(&self.handler)
    }
}
impl SaTokenPlugin for SaTokenPluginForSign {
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
