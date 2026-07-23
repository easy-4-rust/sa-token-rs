//! Wave 3 契约测试：Strategy / Filter / HttpAuth（对应 Java 同名模块）。
//!
//! 作为 `docs/migration/file-map.csv` 中 Wave 3 条目的 `test_evidence`。

use sa_token_core::context::mock::sa_request_for_mock::SaRequestForMock;
use sa_token_core::context::mock::sa_response_for_mock::SaResponseForMock;
use sa_token_core::context::mock::sa_storage_for_mock::SaStorageForMock;
use sa_token_core::context::model::sa_request::SaRequest;
use sa_token_core::context::model::sa_response::SaResponse;
use sa_token_core::context::model::sa_storage::SaStorage;
use sa_token_core::context::sa_token_context::SaTokenContext;
use sa_token_core::context::sa_token_context_for_thread_local::SaTokenContextForThreadLocal;
use sa_token_core::exception::SaTokenException;
use sa_token_core::filter::sa_filter::SaFilter;
use sa_token_core::filter::sa_filter_auth_strategy::SaFilterAuthStrategy;
use sa_token_core::filter::sa_filter_error_strategy::SaFilterErrorStrategy;
use sa_token_core::http::sa_http_template::SaHttpTemplate;
use sa_token_core::http::sa_http_template_default_impl::SaHttpTemplateDefaultImpl;
use sa_token_core::http::sa_http_util::SaHttpUtil;
use sa_token_core::http_auth::{
    SaHttpBasicTemplate, SaHttpBasicUtil, SaHttpDigestModel, SaHttpDigestTemplate,
    SaHttpDigestUtil,
};
use sa_token_core::router::sa_router_staff::SaRouterStaff;
use sa_token_core::sa_manager::SaManager;
use sa_token_core::strategy::hooks::sa_firewall_check_hook::SaFirewallCheckHook;
use sa_token_core::strategy::hooks::sa_firewall_check_hook_for_black_path::SaFirewallCheckHookForBlackPath;
use sa_token_core::strategy::hooks::sa_firewall_check_hook_for_directory_traversal::SaFirewallCheckHookForDirectoryTraversal;
use sa_token_core::strategy::hooks::sa_firewall_check_hook_for_white_path::SaFirewallCheckHookForWhitePath;
use sa_token_core::strategy::sa_firewall_strategy::SaFirewallStrategy;
use sa_token_core::strategy::sa_strategy::SaStrategy;
use std::collections::HashMap;
use std::sync::Arc;

fn set_mock_context(url: &str, method: &str, headers: &[(&str, &str)]) -> Arc<SaResponseForMock> {
    SaManager::set_sa_token_context(Arc::new(SaTokenContextForThreadLocal));
    let mut req = SaRequestForMock::new().with_url(url).with_method(method);
    for (key, value) in headers {
        req = req.with_header(*key, *value);
    }
    let res = Arc::new(SaResponseForMock::new());
    let req: Arc<dyn SaRequest> = Arc::new(req);
    let res_dyn: Arc<dyn SaResponse> = res.clone();
    let stg: Arc<dyn SaStorage> = Arc::new(SaStorageForMock::new());
    SaTokenContextForThreadLocal.set_context(req, res_dyn, stg);
    res
}

struct NoopFilter;

impl SaFilter for NoopFilter {
    fn run(&self, _staff: &SaRouterStaff) {}
}

/// 防火墙：目录遍历检测（对应 Java `SaFirewallCheckHookForDirectoryTraversal`）。
#[test]
fn firewall_directory_traversal_rejects_invalid_path() {
    assert!(!SaFirewallCheckHookForDirectoryTraversal::is_path_valid(
        "/user/../info"
    ));
}

/// 防火墙：白名单命中抛出 StopMatch（对应 Java `SaFirewallCheckHookForWhitePath`）。
#[test]
fn firewall_white_path_stop_match() {
    SaFirewallCheckHookForWhitePath::instance().reset_config(&["/public"]);
    let req = SaRequestForMock::new().with_url("/public");
    let err = SaFirewallCheckHookForWhitePath::instance()
        .execute(&req, &SaResponseForMock::new())
        .unwrap_err();
    assert_eq!(err, SaTokenException::StopMatch);
}

/// 防火墙：黑名单拦截（对应 Java `SaFirewallCheckHookForBlackPath`）。
#[test]
fn firewall_black_path_blocks_request() {
    SaFirewallCheckHookForBlackPath::instance().reset_config(&["/secret"]);
    let req = SaRequestForMock::new().with_url("/secret");
    assert!(
        SaFirewallCheckHookForBlackPath::instance()
            .execute(&req, &SaResponseForMock::new())
            .is_err()
    );
}

/// 防火墙策略链：合法 path 可通过默认 hook（对应 Java `SaFirewallStrategy.check`）。
#[test]
fn firewall_strategy_execute_check_accepts_valid_request() {
    set_mock_context("/user/info", "GET", &[]);
    assert!(SaFirewallStrategy::instance()
        .execute_check(
            &SaRequestForMock::new().with_url("/user/info").with_method("GET"),
            &SaResponseForMock::new()
        )
        .is_ok());
}

/// 全局策略 hasElement 模糊匹配（对应 Java `SaStrategy.hasElement`）。
#[test]
fn sa_strategy_has_element_vague_match() {
    let list = vec!["*.example.com".to_string()];
    assert!((SaStrategy::instance().has_element)(&list, "api.example.com"));
}

/// Filter 策略类型可构造（对应 Java `SaFilterAuthStrategy` / `SaFilterErrorStrategy`）。
#[test]
fn filter_strategy_types_are_callable() {
    let auth: SaFilterAuthStrategy = Box::new(|| {});
    auth();
    let error: SaFilterErrorStrategy = Box::new(|e| e.to_string());
    let _ = error(&SaTokenException::StopMatch);
    let _filter = NoopFilter;
    _filter.run(&SaRouterStaff::new());
}

/// HTTP 默认模板未注入时返回错误（对应 Java `SaHttpTemplateDefaultImpl`）。
#[test]
fn http_default_template_requires_implementation() {
    let template = SaHttpTemplateDefaultImpl;
    assert!(template.get("https://example.com").is_err());
}

/// HTTP 工具默认实现可调用（对应 Java `SaHttpUtil`）。
#[test]
fn http_util_default_get_is_callable() {
    let _ = SaHttpUtil::get("https://example.com");
    let mut params = HashMap::new();
    params.insert("a".to_string(), "1".to_string());
    let _ = SaHttpUtil::post_by_form_data("https://example.com", &params);
}

/// HTTP Basic 校验成功/失败（对应 Java `SaHttpBasicTemplate.check`）。
#[test]
fn http_basic_auth_check_roundtrip() {
    let auth = SaHttpBasicTemplate::build_authorization("sa", "123456");
    set_mock_context("/api", "GET", &[("Authorization", &auth)]);
    assert!(SaHttpBasicUtil::check_with_account("sa:123456").is_ok());

    set_mock_context("/api", "GET", &[]);
    assert!(SaHttpBasicUtil::check_with_account("sa:123456").is_err());
}

/// HTTP Digest 校验成功/失败（对应 Java `SaHttpDigestTemplate.check`）。
#[test]
fn http_digest_auth_check_roundtrip() {
    let mut hope = SaHttpDigestModel::new("sa", "123456");
    hope.nonce = "nonce".to_string();
    hope.uri = "/api".to_string();
    hope.method = "GET".to_string();
    hope.qop = SaHttpDigestModel::DEFAULT_QOP.to_string();
    hope.nc = "00000001".to_string();
    hope.cnonce = "cnonce".to_string();
    hope.opaque = "opaque".to_string();
    hope.response = SaHttpDigestTemplate::calc_response(&hope);
    let auth = SaHttpDigestTemplate::build_authorization(&hope);

    set_mock_context("/api", "GET", &[("Authorization", &auth)]);
    assert!(SaHttpDigestUtil::check_with_account("sa", "123456").is_ok());

    set_mock_context("/api", "GET", &[]);
    assert!(SaHttpDigestUtil::check_with_account("sa", "123456").is_err());
}
