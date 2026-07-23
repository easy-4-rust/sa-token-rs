//! Opt-in serializer plugin marker.

use std::any::Any;

use sa_token_core::plugin::sa_token_plugin::SaTokenPlugin;

/// Plugin marker corresponding to Java `SaTokenPluginForSerializerFeatures`.
///
/// Installation intentionally performs no global registration. Applications
/// explicitly select one serializer and attach it to their runtime.
#[derive(Debug, Default, Clone, Copy)]
pub struct SaTokenPluginForSerializerFeatures;

impl SaTokenPlugin for SaTokenPluginForSerializerFeatures {
    fn install(&self) {}

    fn as_any(&self) -> &dyn Any {
        self
    }
}
