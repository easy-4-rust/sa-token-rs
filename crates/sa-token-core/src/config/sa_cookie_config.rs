//! Cookie write configuration.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

/// Cookie attributes matching Java's nullable configuration model.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SaCookieConfig {
    /// Explicit cookie domain.
    pub domain: Option<String>,
    /// Explicit cookie path.
    pub path: Option<String>,
    /// Whether the cookie is HTTPS-only.
    pub secure: bool,
    /// Whether JavaScript access is disabled.
    pub http_only: bool,
    /// SameSite policy.
    pub same_site: Option<String>,
    /// Extension attributes; `None` represents a flag-only attribute.
    pub extra_attrs: BTreeMap<String, Option<String>>,
}

impl SaCookieConfig {
    /// Sets the cookie domain.
    pub fn set_domain(&mut self, domain: impl Into<String>) -> &mut Self {
        self.domain = Some(domain.into());
        self
    }

    /// Sets the cookie path.
    pub fn set_path(&mut self, path: impl Into<String>) -> &mut Self {
        self.path = Some(path.into());
        self
    }

    /// Sets the secure flag.
    pub fn set_secure(&mut self, secure: bool) -> &mut Self {
        self.secure = secure;
        self
    }

    /// Sets the HttpOnly flag.
    pub fn set_http_only(&mut self, http_only: bool) -> &mut Self {
        self.http_only = http_only;
        self
    }

    /// Sets the SameSite policy.
    pub fn set_same_site(&mut self, same_site: impl Into<String>) -> &mut Self {
        self.same_site = Some(same_site.into());
        self
    }

    /// Replaces all extension attributes.
    pub fn set_extra_attrs(&mut self, extra_attrs: BTreeMap<String, Option<String>>) -> &mut Self {
        self.extra_attrs = extra_attrs;
        self
    }

    /// Adds a valued extension attribute.
    pub fn add_extra_attr(
        &mut self,
        name: impl Into<String>,
        value: impl Into<String>,
    ) -> &mut Self {
        self.extra_attrs.insert(name.into(), Some(value.into()));
        self
    }

    /// Adds a flag-only extension attribute.
    pub fn add_extra_flag(&mut self, name: impl Into<String>) -> &mut Self {
        self.extra_attrs.insert(name.into(), None);
        self
    }

    /// Removes an extension attribute.
    pub fn remove_extra_attr(&mut self, name: &str) -> &mut Self {
        self.extra_attrs.remove(name);
        self
    }
}
