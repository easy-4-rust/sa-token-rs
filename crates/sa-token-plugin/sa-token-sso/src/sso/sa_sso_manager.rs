use crate::sso::config::{SaSsoClientConfig, SaSsoServerConfig};
use crate::sso::error::SaSsoErrorCode;
use crate::sso::exception::SaSsoException;
use std::sync::{Arc, RwLock};

/// Isolated owner of the SSO server and client configuration.
///
/// Unlike the Java static holder, ordinary Rust consumers can construct one
/// manager per runtime or test. A facade may still keep a default instance.
pub struct SaSsoManager {
    server_config: RwLock<Arc<SaSsoServerConfig>>,
    client_config: RwLock<Arc<SaSsoClientConfig>>,
}

impl Default for SaSsoManager {
    fn default() -> Self {
        Self::new(SaSsoServerConfig::default(), SaSsoClientConfig::default())
    }
}

impl SaSsoManager {
    /// Creates an isolated manager with explicit configurations.
    pub fn new(server_config: SaSsoServerConfig, client_config: SaSsoClientConfig) -> Self {
        warn_if_signature_check_disabled(server_config.is_check_sign, "server startup");
        warn_if_signature_check_disabled(client_config.is_check_sign, "client startup");
        Self {
            server_config: RwLock::new(Arc::new(server_config)),
            client_config: RwLock::new(Arc::new(client_config)),
        }
    }

    /// Returns the current server configuration.
    ///
    /// # Errors
    ///
    /// Returns a protocol error when another task poisoned the configuration
    /// lock.
    pub fn server_config(&self) -> Result<Arc<SaSsoServerConfig>, SaSsoException> {
        self.server_config
            .read()
            .map(|config| Arc::clone(&config))
            .map_err(lock_error)
    }

    /// Replaces the server configuration.
    ///
    /// # Errors
    ///
    /// Returns a protocol error when another task poisoned the configuration
    /// lock.
    pub fn set_server_config(&self, config: SaSsoServerConfig) -> Result<(), SaSsoException> {
        warn_if_signature_check_disabled(config.is_check_sign, "startup");
        *self.server_config.write().map_err(lock_error)? = Arc::new(config);
        Ok(())
    }

    /// Returns the current client configuration.
    ///
    /// # Errors
    ///
    /// Returns a protocol error when another task poisoned the configuration
    /// lock.
    pub fn client_config(&self) -> Result<Arc<SaSsoClientConfig>, SaSsoException> {
        self.client_config
            .read()
            .map(|config| Arc::clone(&config))
            .map_err(lock_error)
    }

    /// Replaces the client configuration.
    ///
    /// # Errors
    ///
    /// Returns a protocol error when another task poisoned the configuration
    /// lock.
    pub fn set_client_config(&self, config: SaSsoClientConfig) -> Result<(), SaSsoException> {
        warn_if_signature_check_disabled(config.is_check_sign, "startup");
        *self.client_config.write().map_err(lock_error)? = Arc::new(config);
        Ok(())
    }

    /// Emits the Java-compatible runtime warning through structured logging.
    pub fn warn_signature_check_disabled_at_runtime() {
        warn_if_signature_check_disabled(false, "runtime");
    }
}

fn warn_if_signature_check_disabled(is_enabled: bool, phase: &'static str) {
    if !is_enabled {
        tracing::warn!(
            phase,
            "SSO signature verification is disabled; enable it in production"
        );
    }
}

fn lock_error<T>(_: std::sync::PoisonError<T>) -> SaSsoException {
    SaSsoException::new(
        SaSsoErrorCode::CODE_30001,
        "SSO configuration is unavailable",
    )
}
