use crate::sso::template::{
    SaSsoClientTemplate, SaSsoClientUtil, SaSsoServerTemplate, SaSsoServerUtil,
};

/// Deprecated combined facade retained only as the Java file's mapped type.
///
/// New code should use [`SaSsoClientUtil`] or [`SaSsoServerUtil`] directly.
#[deprecated(note = "use SaSsoClientUtil or SaSsoServerUtil")]
pub struct SaSsoUtil<'a> {
    client: SaSsoClientUtil<'a>,
    server: SaSsoServerUtil<'a>,
}

#[allow(deprecated)]
impl<'a> SaSsoUtil<'a> {
    /// Creates the compatibility facade from explicit templates.
    pub fn new(client: &'a SaSsoClientTemplate, server: &'a SaSsoServerTemplate) -> Self {
        Self {
            client: SaSsoClientUtil::new(client),
            server: SaSsoServerUtil::new(server),
        }
    }

    /// Returns the client facade.
    pub fn client(&self) -> &SaSsoClientUtil<'a> {
        &self.client
    }

    /// Returns the server facade.
    pub fn server(&self) -> &SaSsoServerUtil<'a> {
        &self.server
    }
}
