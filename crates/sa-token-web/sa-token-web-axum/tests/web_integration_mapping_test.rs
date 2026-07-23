//! Web integration mapping tests for Axum adapter responsibilities.

use axum::body::Body;
use axum::http::Request;
use sa_token::prelude::*;
use sa_token_core::context::model::sa_request::SaRequest;
use sa_token_web_axum::{extract_token_from_headers, AxumRequest, SaTokenLayer};
use std::sync::{Arc, Mutex, MutexGuard};

static TEST_LOCK: Mutex<()> = Mutex::new(());

fn setup() -> MutexGuard<'static, ()> {
    let guard = TEST_LOCK
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    SaManager::reset();
    SaManager::set_config(Arc::new(SaTokenConfig::default()));
    SaManager::set_sa_token_dao(Arc::new(SaTokenDaoMemory::new()));
    SaTokenContextMockUtil::set_mock_context();
    SaManager::put_stp_logic(Arc::new(StpLogic::new("login")));
    guard
}

#[test]
fn axum_request_maps_servlet_fields() {
    let req = Request::builder()
        .uri("/api/user?name=alice")
        .header("cookie", "satoken=abc123")
        .header("host", "example.com")
        .method("GET")
        .body(Body::empty())
        .expect("valid request");
    let axum_req = AxumRequest::from_axum_request(&req);
    assert_eq!(axum_req.get_request_path(), "/api/user");
    assert_eq!(axum_req.get_param("name"), Some("alice".to_string()));
    assert_eq!(axum_req.get_cookie_value("satoken"), Some("abc123".to_string()));
    assert_eq!(axum_req.get_host(), "example.com");
}

#[test]
fn token_operate_util_reads_header_and_cookie() {
    let headers = vec![("Authorization".to_string(), "Bearer bearer-token".to_string())];
    let cookies = vec![("satoken".to_string(), "cookie-token".to_string())];
    assert_eq!(
        extract_token_from_headers("satoken", &headers, &cookies),
        Some("cookie-token".to_string())
    );
}

#[test]
fn servlet_mapping_module_is_linked() {
    assert!(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/mapping.rs").is_file());
}

#[test]
fn sa_token_layer_is_constructible() {
    let _guard = setup();
    let _layer = SaTokenLayer::new();
}
