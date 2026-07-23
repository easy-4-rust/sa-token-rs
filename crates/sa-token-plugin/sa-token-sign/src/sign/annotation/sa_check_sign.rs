/// Metadata counterpart of Java `@SaCheckSign`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SaCheckSign {
    pub app_id: String,
    pub verify_params: Vec<String>,
}
