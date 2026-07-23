use std::sync::Arc;

use crate::oauth2::config::SaOAuth2ServerConfig;
use crate::oauth2::dao::SaOAuth2Dao;
use crate::oauth2::data::generate::SaOAuth2DataGenerate;
use crate::oauth2::data::loader::SaOAuth2DataLoader;
use crate::oauth2::data::resolver::SaOAuth2DataResolver;

/// Complete isolated OAuth2 component graph.
pub struct SaOAuth2Runtime<C> {
    pub config: Arc<SaOAuth2ServerConfig>,
    pub loader: Arc<dyn SaOAuth2DataLoader>,
    pub resolver: Arc<dyn SaOAuth2DataResolver>,
    pub converter: Arc<C>,
    pub generate: Arc<dyn SaOAuth2DataGenerate>,
    pub dao: Arc<SaOAuth2Dao>,
}

impl<C> SaOAuth2Runtime<C> {
    pub fn new(
        config: Arc<SaOAuth2ServerConfig>,
        loader: Arc<dyn SaOAuth2DataLoader>,
        resolver: Arc<dyn SaOAuth2DataResolver>,
        converter: Arc<C>,
        generate: Arc<dyn SaOAuth2DataGenerate>,
        dao: Arc<SaOAuth2Dao>,
    ) -> Self {
        Self {
            config,
            loader,
            resolver,
            converter,
            generate,
            dao,
        }
    }
}

/// Runtime holder replacing Java's mutable component-by-component globals.
pub struct SaOAuth2Manager<C> {
    runtime: Arc<SaOAuth2Runtime<C>>,
}

impl<C> Clone for SaOAuth2Manager<C> {
    fn clone(&self) -> Self {
        Self {
            runtime: Arc::clone(&self.runtime),
        }
    }
}

impl<C> SaOAuth2Manager<C> {
    pub fn new(runtime: Arc<SaOAuth2Runtime<C>>) -> Self {
        Self { runtime }
    }

    pub fn runtime(&self) -> &Arc<SaOAuth2Runtime<C>> {
        &self.runtime
    }

    pub fn server_config(&self) -> &Arc<SaOAuth2ServerConfig> {
        &self.runtime.config
    }

    pub fn data_loader(&self) -> &Arc<dyn SaOAuth2DataLoader> {
        &self.runtime.loader
    }

    pub fn data_resolver(&self) -> &Arc<dyn SaOAuth2DataResolver> {
        &self.runtime.resolver
    }

    pub fn data_converter(&self) -> &Arc<C> {
        &self.runtime.converter
    }

    pub fn data_generate(&self) -> &Arc<dyn SaOAuth2DataGenerate> {
        &self.runtime.generate
    }

    pub fn dao(&self) -> &Arc<SaOAuth2Dao> {
        &self.runtime.dao
    }
}
