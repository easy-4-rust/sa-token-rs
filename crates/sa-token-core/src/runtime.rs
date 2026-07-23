//! Explicit, isolated Sa-Token runtime containers.

use std::sync::Arc;

use crate::config::sa_token_config::SaTokenConfig;
use crate::context::sa_token_context::SaTokenContext;
use crate::dao::{AsyncSaTokenDao, SaTokenDao};
use crate::listener::SaListenerTrait as SaTokenListener;
use crate::plugin::sa_token_plugin::SaTokenPlugin;
use crate::stp::{StpInterface, StpInterfaceDefaultImpl};

/// Dependencies for an isolated synchronous Sa-Token execution context.
pub struct SaTokenRuntime {
    config: Arc<SaTokenConfig>,
    dao: Arc<dyn SaTokenDao>,
    context: Arc<dyn SaTokenContext>,
    stp_interface: Arc<dyn StpInterface>,
    listeners: Vec<Arc<dyn SaTokenListener>>,
    plugins: Vec<Arc<dyn SaTokenPlugin>>,
}

impl SaTokenRuntime {
    /// Creates an isolated runtime with explicit core dependencies.
    pub fn new(
        config: Arc<SaTokenConfig>,
        dao: Arc<dyn SaTokenDao>,
        context: Arc<dyn SaTokenContext>,
    ) -> Self {
        Self {
            config,
            dao,
            context,
            stp_interface: Arc::new(StpInterfaceDefaultImpl),
            listeners: Vec::new(),
            plugins: Vec::new(),
        }
    }

    /// Replaces the permission and role provider.
    pub fn with_stp_interface(mut self, stp_interface: Arc<dyn StpInterface>) -> Self {
        self.stp_interface = stp_interface;
        self
    }

    /// Registers a listener owned by this runtime.
    pub fn with_listener(mut self, listener: Arc<dyn SaTokenListener>) -> Self {
        self.listeners.push(listener);
        self
    }

    /// Registers a plugin owned by this runtime.
    pub fn with_plugin(mut self, plugin: Arc<dyn SaTokenPlugin>) -> Self {
        self.plugins.push(plugin);
        self
    }

    /// Returns the immutable runtime configuration.
    pub fn config(&self) -> &Arc<SaTokenConfig> {
        &self.config
    }

    /// Returns the synchronous persistence port.
    pub fn dao(&self) -> &Arc<dyn SaTokenDao> {
        &self.dao
    }

    /// Returns the request context port.
    pub fn context(&self) -> &Arc<dyn SaTokenContext> {
        &self.context
    }

    /// Returns the permission and role provider.
    pub fn stp_interface(&self) -> &Arc<dyn StpInterface> {
        &self.stp_interface
    }

    /// Returns runtime-local listeners.
    pub fn listeners(&self) -> &[Arc<dyn SaTokenListener>] {
        &self.listeners
    }

    /// Returns runtime-local plugins.
    pub fn plugins(&self) -> &[Arc<dyn SaTokenPlugin>] {
        &self.plugins
    }
}

/// Dependencies for an isolated asynchronous Sa-Token execution context.
pub struct AsyncSaTokenRuntime {
    config: Arc<SaTokenConfig>,
    dao: Arc<dyn AsyncSaTokenDao>,
    context: Arc<dyn SaTokenContext>,
    stp_interface: Arc<dyn StpInterface>,
    listeners: Vec<Arc<dyn SaTokenListener>>,
    plugins: Vec<Arc<dyn SaTokenPlugin>>,
}

impl AsyncSaTokenRuntime {
    /// Creates an isolated asynchronous runtime with an object-safe async DAO.
    pub fn new(
        config: Arc<SaTokenConfig>,
        dao: Arc<dyn AsyncSaTokenDao>,
        context: Arc<dyn SaTokenContext>,
    ) -> Self {
        Self {
            config,
            dao,
            context,
            stp_interface: Arc::new(StpInterfaceDefaultImpl),
            listeners: Vec::new(),
            plugins: Vec::new(),
        }
    }

    /// Replaces the permission and role provider.
    pub fn with_stp_interface(mut self, stp_interface: Arc<dyn StpInterface>) -> Self {
        self.stp_interface = stp_interface;
        self
    }

    /// Registers a listener owned by this runtime.
    pub fn with_listener(mut self, listener: Arc<dyn SaTokenListener>) -> Self {
        self.listeners.push(listener);
        self
    }

    /// Registers a plugin owned by this runtime.
    pub fn with_plugin(mut self, plugin: Arc<dyn SaTokenPlugin>) -> Self {
        self.plugins.push(plugin);
        self
    }

    /// Returns the immutable runtime configuration.
    pub fn config(&self) -> &Arc<SaTokenConfig> {
        &self.config
    }

    /// Returns the asynchronous persistence port.
    pub fn dao(&self) -> &Arc<dyn AsyncSaTokenDao> {
        &self.dao
    }

    /// Returns the request context port.
    pub fn context(&self) -> &Arc<dyn SaTokenContext> {
        &self.context
    }

    /// Returns the permission and role provider.
    pub fn stp_interface(&self) -> &Arc<dyn StpInterface> {
        &self.stp_interface
    }

    /// Returns runtime-local listeners.
    pub fn listeners(&self) -> &[Arc<dyn SaTokenListener>] {
        &self.listeners
    }

    /// Returns runtime-local plugins.
    pub fn plugins(&self) -> &[Arc<dyn SaTokenPlugin>] {
        &self.plugins
    }
}
