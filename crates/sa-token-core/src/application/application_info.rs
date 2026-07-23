//! Application-wide route-prefix metadata.

use std::sync::{OnceLock, RwLock};

/// Application metadata shared by framework adapters.
pub struct ApplicationInfo;

impl ApplicationInfo {
    fn route_prefix_cell() -> &'static RwLock<String> {
        static ROUTE_PREFIX: OnceLock<RwLock<String>> = OnceLock::new();
        ROUTE_PREFIX.get_or_init(|| RwLock::new(String::new()))
    }

    /// Replaces the application route prefix.
    pub fn set_route_prefix(prefix: impl Into<String>) {
        *Self::route_prefix_cell()
            .write()
            .unwrap_or_else(|poisoned| poisoned.into_inner()) = prefix.into();
    }

    /// Returns the configured application route prefix.
    pub fn route_prefix() -> String {
        Self::route_prefix_cell()
            .read()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clone()
    }

    /// Removes the configured non-root prefix from a request path.
    pub fn cut_path_prefix(path: &str) -> String {
        let prefix = Self::route_prefix();
        if !prefix.is_empty() && prefix != "/" {
            path.strip_prefix(&prefix).unwrap_or(path).to_owned()
        } else {
            path.to_owned()
        }
    }
}
