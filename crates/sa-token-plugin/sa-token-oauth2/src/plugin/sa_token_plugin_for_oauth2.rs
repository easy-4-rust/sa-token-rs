use std::any::Any;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use sa_token_core::plugin::sa_token_plugin::SaTokenPlugin;

use crate::oauth2::annotation::SaOAuth2AnnotationValidator;
use crate::oauth2::annotation::handler::{
    SaCheckAccessTokenHandler, SaCheckClientIdSecretHandler, SaCheckClientTokenHandler,
};

/// Registration port used by framework adapters to expose OAuth2 annotations.
///
/// Each runtime owns its registry. This keeps plugin installation isolated and
/// avoids mutating Java-style process-global annotation state.
pub trait SaOAuth2AnnotationRegistry<V>: Send + Sync + 'static
where
    V: SaOAuth2AnnotationValidator,
{
    /// Registers the access-token annotation handler.
    fn register_access_token_handler(&self, handler: Arc<SaCheckAccessTokenHandler<V>>);

    /// Registers the client-token annotation handler.
    fn register_client_token_handler(&self, handler: Arc<SaCheckClientTokenHandler<V>>);

    /// Registers the client credential annotation handler.
    fn register_client_id_secret_handler(&self, handler: Arc<SaCheckClientIdSecretHandler<V>>);

    /// Removes all handlers registered by this OAuth2 plugin instance.
    fn unregister_oauth2_handlers(&self);
}

/// Installs the three OAuth2 annotation handlers into an explicit registry.
pub struct SaTokenPluginForOAuth2<V, R>
where
    V: SaOAuth2AnnotationValidator,
    R: SaOAuth2AnnotationRegistry<V>,
{
    installed: AtomicBool,
    registry: Arc<R>,
    access_token_handler: Arc<SaCheckAccessTokenHandler<V>>,
    client_token_handler: Arc<SaCheckClientTokenHandler<V>>,
    client_id_secret_handler: Arc<SaCheckClientIdSecretHandler<V>>,
}

impl<V, R> SaTokenPluginForOAuth2<V, R>
where
    V: SaOAuth2AnnotationValidator,
    R: SaOAuth2AnnotationRegistry<V>,
{
    /// Creates a plugin whose handlers all use the same runtime validator.
    pub fn new(validator: Arc<V>, registry: Arc<R>) -> Self {
        Self {
            installed: AtomicBool::new(false),
            registry,
            access_token_handler: Arc::new(SaCheckAccessTokenHandler::new(Arc::clone(&validator))),
            client_token_handler: Arc::new(SaCheckClientTokenHandler::new(Arc::clone(&validator))),
            client_id_secret_handler: Arc::new(SaCheckClientIdSecretHandler::new(validator)),
        }
    }

    /// Returns whether this plugin instance is installed.
    pub fn is_installed(&self) -> bool {
        self.installed.load(Ordering::Acquire)
    }
}

impl<V, R> SaTokenPlugin for SaTokenPluginForOAuth2<V, R>
where
    V: SaOAuth2AnnotationValidator,
    R: SaOAuth2AnnotationRegistry<V>,
{
    fn install(&self) {
        if self
            .installed
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .is_err()
        {
            return;
        }

        self.registry
            .register_access_token_handler(Arc::clone(&self.access_token_handler));
        self.registry
            .register_client_token_handler(Arc::clone(&self.client_token_handler));
        self.registry
            .register_client_id_secret_handler(Arc::clone(&self.client_id_secret_handler));
    }

    fn destroy(&self) {
        if self
            .installed
            .compare_exchange(true, false, Ordering::AcqRel, Ordering::Acquire)
            .is_ok()
        {
            self.registry.unregister_oauth2_handlers();
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
