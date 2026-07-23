/// Requires a valid access token containing every requested scope.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SaCheckAccessToken {
    pub scope: Vec<String>,
}
