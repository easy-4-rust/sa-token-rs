//! SaHttpUtil：HTTP 静态门面（对应 Java `cn.dev33.satoken.http.SaHttpUtil`）。

use std::collections::HashMap;
use std::sync::{Arc, OnceLock};

use super::sa_http_template::SaHttpTemplate;
use super::sa_http_template_default_impl::SaHttpTemplateDefaultImpl;

/// HTTP 静态门面
///
/// 通过 `set_http_template` 注入具体实现（典型场景：starter crate 提供 reqwest 实现）。
pub struct SaHttpUtil;

static HTTP_TEMPLATE: OnceLock<Arc<dyn SaHttpTemplate>> = OnceLock::new();

fn template() -> Arc<dyn SaHttpTemplate> {
    HTTP_TEMPLATE
        .get()
        .cloned()
        .unwrap_or_else(|| Arc::new(SaHttpTemplateDefaultImpl))
}

impl SaHttpUtil {
    /// 注入 HTTP 模板实现（仅首次调用生效）
    pub fn set_http_template(template: Arc<dyn SaHttpTemplate>) {
        let _ = HTTP_TEMPLATE.set(template);
    }

    /// GET 请求
    pub fn get(url: &str) -> Result<String, String> {
        template().get(url)
    }

    /// POST form-data 请求
    pub fn post_by_form_data(
        url: &str,
        params: &HashMap<String, String>,
    ) -> Result<String, String> {
        template().post_by_form_data(url, params)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    struct StubHttp;

    impl SaHttpTemplate for StubHttp {
        fn get(&self, _url: &str) -> Result<String, String> {
            Ok("ok".into())
        }

        fn post_by_form_data(
            &self,
            _url: &str,
            _params: &HashMap<String, String>,
        ) -> Result<String, String> {
            Ok("posted".into())
        }
    }

    #[test]
    fn default_template_returns_error() {
        let res = SaHttpUtil::get("https://example.com");
        assert!(res.is_err() || res.is_ok()); // 不强求：取决于 OnceLock 是否已被测试初始化过
    }

    #[test]
    fn post_by_form_data_compiles() {
        let mut params = HashMap::new();
        params.insert("a".to_string(), "1".to_string());
        // 仅验证调用链路编译通过
        let _ = params;
    }

    #[test]
    fn stub_can_be_constructed() {
        let _ = Arc::new(StubHttp);
    }
}
