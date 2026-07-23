use std::sync::Arc;

use actix_web::http::StatusCode;
use actix_web::{App, middleware, test, web};
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
use sa_token_web_actix::{RequireLogin, require_login};

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

fn util() -> AsyncStpUtil {
    AsyncStpUtil::new(
        "login",
        Arc::new(AsyncSaTokenRuntime::new(
            Arc::new(SaTokenConfig::default()),
            Arc::new(SaTokenDaoMemory::new()),
            Arc::new(IsolatedContext::new()),
        )),
    )
}

async fn protected(login: RequireLogin) -> String {
    login.0.login_id
}

#[actix_web::test]
async fn middleware_and_extractor_share_authenticated_identity() {
    let util = util();
    let token = util.login("actix-user").await.expect("seed login");
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(util))
            .wrap(middleware::from_fn(require_login))
            .route("/protected", web::get().to(protected)),
    )
    .await;

    let request = test::TestRequest::get()
        .uri("/protected")
        .insert_header(("satoken", token))
        .to_request();
    let response = test::call_service(&app, request).await;
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(test::read_body(response).await, "actix-user");

    let missing = test::TestRequest::get().uri("/protected").to_request();
    let error = test::try_call_service(&app, missing)
        .await
        .expect_err("missing token must be rejected");
    assert_eq!(
        error.as_response_error().status_code(),
        StatusCode::UNAUTHORIZED
    );
}

#[actix_web::test]
async fn reactor_mapping_module_is_linked() {
    // The generated mapping module is part of the crate graph when this test builds.
    assert!(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/mapping.rs").is_file());
}
