//! `SaHttpTemplateDefaultImpl` —— 1:1 对应 Java `cn.dev33.satoken.http.SaHttpTemplateDefaultImpl`

use std::collections::HashMap;

use super::sa_http_template::SaHttpTemplate;

/// 默认 HTTP 模板：未实现（需要外部 starter 注入）
pub struct SaHttpTemplateDefaultImpl;

impl Default for SaHttpTemplateDefaultImpl {
    fn default() -> Self {
        Self
    }
}

impl SaHttpTemplate for SaHttpTemplateDefaultImpl {
    fn get(&self, _url: &str) -> Result<String, String> {
        Err("请配置 SaHttpTemplate 实现".to_string())
    }

    fn post_by_form_data(
        &self,
        _url: &str,
        _params: &HashMap<String, String>,
    ) -> Result<String, String> {
        Err("请配置 SaHttpTemplate 实现".to_string())
    }
}
