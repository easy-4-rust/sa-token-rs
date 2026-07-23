/// Request extension populated after successful authentication.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LoginIdentity {
    /// Resolved login id.
    pub login_id: String,
    /// Token used for authentication.
    pub token: String,
}
