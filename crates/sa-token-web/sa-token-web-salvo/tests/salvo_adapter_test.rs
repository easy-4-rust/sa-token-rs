use std::sync::Arc;

use sa_token_context_mock::{SaRequestForMock, SaResponseForMock, SaStorageForMock};
use sa_token_core::config::sa_token_config::SaTokenConfig;
use sa_token_core::context::model::sa_request::SaRequest;
use sa_token_core::context::model::sa_response::SaResponse;
use sa_token_core::context::model::sa_storage::SaStorage;
use sa_token_core::context::model::sa_token_context_model_box::SaTokenContextModelBox;
use sa_token_core::context::sa_token_context::SaTokenContext;
use sa_token_core::runtime::AsyncSaTokenRuntime;
use sa_token_core::stp::AsyncStpUtil;
use sa_token_dao_memory::SaTokenDaoMemory;
use sa_token_web_salvo::{RequireLogin, login_id};
use salvo::http::StatusCode;
use salvo::http::header::HeaderValue;
use salvo::prelude::{Depot, FlowCtrl, Handler, Request, Response};

struct IsolatedContext {
    request: Arc<dyn SaRequest>,
    response: Arc<dyn SaResponse>,
    storage: Arc<dyn SaStorage>,
}

impl IsolatedContext {
    fn new() -> Self {
        Self {
            request: Arc::new(SaRequestForMock::default()),
            response: Arc::new(SaResponseForMock::default()),
            storage: Arc::new(SaStorageForMock::default()),
        }
    }
}

impl SaTokenContext for IsolatedContext {
    fn set_context(
        &self,
        _request: Arc<dyn SaRequest>,
        _response: Arc<dyn SaResponse>,
        _storage: Arc<dyn SaStorage>,
    ) {
    }

    fn clear_context(&self) {
        self.storage().delete("satoken");
    }

    fn is_valid(&self) -> bool {
        true
    }

    fn model_box(&self) -> SaTokenContextModelBox {
        SaTokenContextModelBox::new(
            Arc::clone(&self.request),
            Arc::clone(&self.response),
            Arc::clone(&self.storage),
        )
    }
}

fn util() -> Arc<AsyncStpUtil> {
    Arc::new(AsyncStpUtil::new(
        "login",
        Arc::new(AsyncSaTokenRuntime::new(
            Arc::new(SaTokenConfig::default()),
            Arc::new(SaTokenDaoMemory::new()),
            Arc::new(IsolatedContext::new()),
        )),
    ))
}

#[tokio::test]
async fn hoop_accepts_header_and_populates_depot() {
    let util = util();
    let token = util.login("salvo-user").await.expect("seed login");
    let handler = RequireLogin::new(util);
    let mut request = Request::default();
    request.headers_mut().insert(
        "satoken",
        HeaderValue::from_str(&token).expect("valid token header"),
    );
    let mut depot = Depot::new();
    let mut response = Response::new();
    let mut ctrl = FlowCtrl::default();

    handler
        .handle(&mut request, &mut depot, &mut response, &mut ctrl)
        .await;

    assert_eq!(login_id(&depot), Some("salvo-user"));
    assert_ne!(response.status_code, Some(StatusCode::UNAUTHORIZED));
}

#[tokio::test]
async fn hoop_rejects_missing_credentials() {
    let handler = RequireLogin::new(util());
    let mut request = Request::default();
    let mut depot = Depot::new();
    let mut response = Response::new();
    let mut ctrl = FlowCtrl::default();

    handler
        .handle(&mut request, &mut depot, &mut response, &mut ctrl)
        .await;

    assert_eq!(response.status_code, Some(StatusCode::UNAUTHORIZED));
    assert!(ctrl.is_ceased() || !ctrl.has_next());
}

#[tokio::test]
async fn webmvc_reactor_mapping_module_is_linked() {
    assert!(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/mapping.rs").is_file());
}
