//! `SaHttpTemplate` —— 1:1 对应 Java `cn.dev33.satoken.http.SaHttpTemplate`

use std::collections::HashMap;

/// HTTP 调用模板
///
/// 对应 Java `SaHttpTemplate`。具体实现由 Web 框架 starter 提供
/// （例如基于 reqwest / ureq 的实现）。
pub trait SaHttpTemplate: Send + Sync + 'static {
    /// GET 请求
    fn get(&self, url: &str) -> Result<String, String>;

    /// POST form-data 请求
    fn post_by_form_data(
        &self,
        url: &str,
        params: &HashMap<String, String>,
    ) -> Result<String, String>;
}
