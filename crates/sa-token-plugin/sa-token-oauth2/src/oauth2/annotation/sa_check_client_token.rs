/// Requires a valid client token containing every requested scope.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SaCheckClientToken {
    pub scope: Vec<String>,
}
