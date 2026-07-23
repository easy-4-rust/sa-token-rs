//! Quick login plugin contract tests.

use sa_token_core::config::sa_token_config::SaTokenConfig;
use sa_token_core::context::mock::sa_request_for_mock::SaRequestForMock;
use sa_token_core::context::mock::sa_response_for_mock::SaResponseForMock;
use sa_token_core::context::mock::sa_storage_for_mock::SaStorageForMock;
use sa_token_core::context::model::sa_token_context_model_box::SaTokenContextModelBox;
use sa_token_core::context::model::sa_request::SaRequest;
use sa_token_core::context::model::sa_response::SaResponse;
use sa_token_core::context::model::sa_storage::SaStorage;
use sa_token_core::context::sa_token_context::SaTokenContext;
use sa_token_core::exception::SaTokenException;
use sa_token_core::sa_manager::SaManager;
use sa_token_core::stp::stp_logic::StpLogic;
use sa_token_dao_memory::SaTokenDaoMemory;
use sa_token_quick_login::{
    SaQuickConfig, SaQuickController, SaQuickInject, SaQuickManager, SaQuickRegister,
};
use std::sync::{Arc, Mutex, MutexGuard};

static TEST_LOCK: Mutex<()> = Mutex::new(());

struct TestContext {
    request: Arc<dyn SaRequest>,
    response: Arc<dyn SaResponse>,
    storage: Arc<dyn SaStorage>,
}

impl SaTokenContext for TestContext {
    fn set_context(
        &self,
        _request: Arc<dyn SaRequest>,
        _response: Arc<dyn SaResponse>,
        _storage: Arc<dyn SaStorage>,
    ) {
    }

    fn clear_context(&self) {}

    fn is_valid(&self) -> bool {
        true
    }

    fn request(&self) -> Arc<dyn SaRequest> {
        Arc::clone(&self.request)
    }

    fn response(&self) -> Arc<dyn SaResponse> {
        Arc::clone(&self.response)
    }

    fn storage(&self) -> Arc<dyn SaStorage> {
        Arc::clone(&self.storage)
    }

    fn model_box(&self) -> SaTokenContextModelBox {
        SaTokenContextModelBox::new(
            Arc::clone(&self.request),
            Arc::clone(&self.response),
            Arc::clone(&self.storage),
        )
    }
}

fn setup(path: &str) -> MutexGuard<'static, ()> {
    let guard = TEST_LOCK
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    SaManager::reset();
    SaManager::set_config(Arc::new(SaTokenConfig::default()));
    SaManager::set_sa_token_dao(Arc::new(SaTokenDaoMemory::new()));
    let req = SaRequestForMock::default().with_url(path);
    SaManager::set_sa_token_context(Arc::new(TestContext {
        request: Arc::new(req),
        response: Arc::new(SaResponseForMock::new()),
        storage: Arc::new(SaStorageForMock::new()),
    }));
    SaManager::put_stp_logic(Arc::new(StpLogic::new("login")));
    SaQuickInject::inject(Some(SaQuickRegister::default_config()));
    guard
}

#[test]
fn controller_do_login_matches_config_credentials() {
    let _guard = setup("/doLogin");
    let ok = SaQuickController::do_login("sa", "123456");
    assert!(ok.is_ok());
    let bad = SaQuickController::do_login("sa", "bad");
    assert!(!bad.is_ok());
}

#[test]
fn register_auth_blocks_protected_route() {
    let _guard = setup("/admin/list");
    let err = SaQuickRegister::auth_check().expect_err("must forward login");
    assert!(matches!(err, SaTokenException::BackResult { .. }));
}

#[test]
fn manager_auto_mode_generates_credentials() {
    let mut cfg = SaQuickConfig::default();
    cfg.auto = true;
    SaQuickManager::set_config(cfg);
    let cfg = SaQuickManager::get_config();
    assert_eq!(cfg.name.len(), 8);
    assert_eq!(cfg.pwd.len(), 8);
}
