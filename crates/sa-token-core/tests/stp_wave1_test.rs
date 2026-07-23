//! Wave 1 STP / Session / Temp 契约测试（对应 Java StpLogic 阶梯封禁、共享 Token、匿名 Token-Session）。

use std::sync::{Arc, Mutex, MutexGuard};

use sa_token_context_mock::{SaRequestForMock, SaResponseForMock, SaStorageForMock, SaTokenContextMockUtil};
use sa_token_core::config::sa_token_config::SaTokenConfig;
use sa_token_core::context::model::sa_request::SaRequest;
use sa_token_core::context::model::sa_response::SaResponse;
use sa_token_core::context::model::sa_storage::SaStorage;
use sa_token_core::context::model::sa_token_context_model_box::SaTokenContextModelBox;
use sa_token_core::context::sa_token_context::SaTokenContext;
use sa_token_core::error::SaErrorCode;
use sa_token_core::exception::SaTokenException;
use sa_token_core::model::wrapper_info::sa_disable_wrapper_info::SaDisableWrapperInfo;
use sa_token_core::runtime::AsyncSaTokenRuntime;
use sa_token_core::sa_manager::SaManager;
use sa_token_core::session::raw::sa_raw_session_delegator::SaRawSessionDelegator;
use sa_token_core::session::raw::sa_raw_session_util::SaRawSessionUtil;
use sa_token_core::session::sa_session::SaSession;
use sa_token_core::session::sa_session_custom_util::SaSessionCustomUtil;
use sa_token_core::session::sa_terminal_info::SaTerminalInfo;
use sa_token_core::stp::parameter::enums::{
    sa_logout_mode::SaLogoutMode, sa_logout_range::SaLogoutRange,
    sa_replaced_login_exit_mode::SaReplacedLoginExitMode, sa_replaced_range::SaReplacedRange,
};
use sa_token_core::stp::parameter::sa_login_parameter::SaLoginParameter;
use sa_token_core::stp::parameter::sa_logout_parameter::SaLogoutParameter;
use sa_token_core::stp::sa_login_config::SaLoginConfig;
use sa_token_core::stp::sa_login_model::SaLoginModel;
use sa_token_core::stp::sa_token_info::SaTokenInfo;
use sa_token_core::stp::stp_interface::StpInterface;
use sa_token_core::stp::stp_logic::StpLogic;
use sa_token_core::stp::stp_util::StpUtil;
use sa_token_core::stp::{AsyncStpUtil, StpInterfaceDefaultImpl};
use sa_token_core::temp::sa_temp_template::SaTempTemplateDefault;
use sa_token_core::temp::sa_temp_util::SaTempUtil;
use sa_token_core::util::sa_token_consts::{
    DEFAULT_DISABLE_LEVEL, MIN_DISABLE_LEVEL, NOT_DISABLE_LEVEL, SESSION_TYPE_ANON,
};
use sa_token_dao_memory::SaTokenDaoMemory;

static TEST_LOCK: Mutex<()> = Mutex::new(());

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

    fn clear_context(&self) {}

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

struct LevelDisableProvider;

impl StpInterface for LevelDisableProvider {
    fn get_permission_list(&self, _login_id: &str, _login_type: &str) -> Vec<String> {
        Vec::new()
    }

    fn get_role_list(&self, _login_id: &str, _login_type: &str) -> Vec<String> {
        Vec::new()
    }

    fn is_disabled(&self, login_id: &str, service: &str) -> SaDisableWrapperInfo {
        if login_id == "loader" && service == "comment" {
            SaDisableWrapperInfo::create_disabled(30, 3)
        } else {
            SaDisableWrapperInfo::create_not_disabled()
        }
    }
}

fn setup_sync() -> MutexGuard<'static, ()> {
    let guard = TEST_LOCK
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    SaManager::reset();
    SaManager::set_config(Arc::new(SaTokenConfig::default()));
    SaManager::set_sa_token_dao(Arc::new(SaTokenDaoMemory::new()));
    SaManager::set_stp_interface(Arc::new(LevelDisableProvider));
    SaManager::set_sa_temp_template(Arc::new(SaTempTemplateDefault::default()));
    SaTokenContextMockUtil::set_mock_context();
    SaManager::put_stp_logic(Arc::new(StpLogic::new("login")));
    guard
}

fn async_runtime() -> Arc<AsyncSaTokenRuntime> {
    Arc::new(
        AsyncSaTokenRuntime::new(
            Arc::new(SaTokenConfig::default()),
            Arc::new(SaTokenDaoMemory::new()),
            Arc::new(IsolatedContext::new()),
        )
        .with_stp_interface(Arc::new(LevelDisableProvider)),
    )
}

#[test]
fn wave1_models_parameters_and_enums_defaults() {
    let _guard = setup_sync();

    let config = SaLoginConfig::default()
        .with_timeout(120)
        .with_share(true)
        .with_device_type("mobile");
    assert_eq!(config.timeout, 120);
    assert!(config.is_share);

    let model = SaLoginModel::new("10001").with_config(config.clone());
    assert_eq!(model.login_id, "10001");
    assert!(model.config.is_some());

    let info = SaTokenInfo::new("satoken", "tok");
    assert_eq!(info.token_name, "satoken");
    assert!(info.is_created);

    let login_param = SaLoginParameter::default()
        .set_device_type("pc")
        .set_is_share(true);
    assert_eq!(login_param.get_is_share(&SaTokenConfig::default()), true);

    let logout_param = SaLogoutParameter::create()
        .set_mode(SaLogoutMode::Kickout)
        .set_range(SaLogoutRange::Account);
    assert_eq!(logout_param.mode, SaLogoutMode::Kickout);
    assert_eq!(logout_param.range, SaLogoutRange::Account);
    assert_eq!(SaReplacedLoginExitMode::default(), SaReplacedLoginExitMode::OldDeviceOffline);
    assert_eq!(SaReplacedRange::default(), SaReplacedRange::CurrDeviceType);

    let default_impl = StpInterfaceDefaultImpl;
    assert!(default_impl.get_role_list("1", "login").is_empty());
    assert_eq!(
        default_impl.is_disabled("1", "login"),
        SaDisableWrapperInfo::create_not_disabled()
    );
}

#[test]
fn wave1_sync_share_token_and_disable_level() {
    let _guard = setup_sync();
    let logic = StpLogic::new("login");
    let shared_param = SaLoginParameter::default()
        .set_device_type("desktop")
        .set_is_share(true);

    let first = logic
        .create_login_session("alice", &shared_param)
        .expect("first shared login");
    let second = logic
        .create_login_session("alice", &shared_param)
        .expect("second shared login");
    assert_eq!(first, second);
    assert_eq!(
        logic
            .get_terminal_list_by_login_id("alice")
            .expect("terminal list")
            .len(),
        1
    );

    logic
        .disable_level("alice", 2, 60)
        .expect("disable level 2");
    assert_eq!(
        logic.get_disable_level("alice").expect("disable level"),
        2
    );
    assert!(logic.is_disable_level("alice", 2).expect("is disable"));
    assert!(!logic.is_disable_level("alice", 3).expect("below level"));
    assert!(logic.check_disable_level("alice", 2).is_err());

    logic.untie_disable("alice").expect("untie");
    assert_eq!(
        logic.get_disable_level("alice").expect("not disabled"),
        NOT_DISABLE_LEVEL
    );

    assert_eq!(
        logic
            .get_disable_level_with_service("loader", "comment")
            .expect("loader disable"),
        3
    );
}

#[test]
fn wave1_sync_anon_token_session_and_terminal_info() {
    let _guard = setup_sync();
    let logic = StpLogic::new("login");

    let anon = logic.get_anon_token_session().expect("anon session");
    assert_eq!(anon.session_type(), SESSION_TYPE_ANON);
    assert!(!anon.token().is_empty());

    let terminal = SaTerminalInfo::new(1, "tok", "pc");
    assert_eq!(terminal.device_type(), "pc");
    assert_eq!(terminal.token_value(), "tok");
}

#[test]
fn wave1_session_raw_and_temp_util() {
    let _guard = setup_sync();

    let custom = SaSessionCustomUtil::get_session_by_id("custom-1").expect("custom session");
    assert_eq!(custom.id(), "custom-1");

    let raw = SaRawSessionUtil::get_session("raw-1").expect("raw session");
    assert_eq!(raw.id(), "raw-1");
    SaRawSessionUtil::save_session(&raw, 30).expect("save raw");

    let delegator = SaRawSessionDelegator::get_session("delegator-1").expect("delegator get");
    SaRawSessionDelegator::save_session(&delegator, 30).expect("delegator save");
    SaRawSessionDelegator::delete_session("delegator-1").expect("delegator delete");

    let value = serde_json::json!({"k": "v"});
    let token = SaTempUtil::create_token(&value, 60).expect("temp token");
    assert_eq!(
        SaTempUtil::parse_token(&token)
            .expect("parse temp")
            .expect("temp value"),
        value
    );
    SaTempUtil::delete_token(&token).expect("delete temp");
}

#[test]
fn wave1_stp_util_facade_exposes_new_apis() {
    let _guard = setup_sync();

    StpUtil::login("10001").expect("login");
    let _anon = StpUtil::get_anon_token_session().expect("anon via facade");
    StpUtil::disable_level("10001", DEFAULT_DISABLE_LEVEL, 30).expect("disable level");
    assert_eq!(
        StpUtil::get_disable_level("10001").expect("level"),
        DEFAULT_DISABLE_LEVEL
    );
    assert!(StpUtil::is_disable_level("10001", MIN_DISABLE_LEVEL).expect("is disable"));
}

#[tokio::test]
async fn wave1_async_disable_level_and_anon_session() {
    let util = AsyncStpUtil::new("login", async_runtime());
    util.disable_level("bob", 2, 60)
        .await
        .expect("async disable level");
    assert_eq!(util.get_disable_level("bob").await.expect("level"), 2);
    assert!(util.check_disable_level("bob", 2).await.is_err());

    let anon = util.get_anon_token_session().await.expect("async anon");
    assert_eq!(anon.session_type(), SESSION_TYPE_ANON);
}

#[test]
fn wave1_disable_level_validation_uses_java_error_codes() {
    let _guard = setup_sync();
    let logic = StpLogic::new("login");

    let err = logic
        .disable_level_with_service("", "login", 1, 10)
        .expect_err("empty login id");
    assert_eq!(err.code(), SaErrorCode::CODE_11062);

    let err = logic
        .disable_level_with_service("u1", "", 1, 10)
        .expect_err("empty service");
    assert_eq!(err.code(), SaErrorCode::CODE_11063);

    let err = logic
        .disable_level("u1", -1, 10)
        .expect_err("invalid level");
    assert_eq!(err.code(), SaErrorCode::CODE_11064);

    logic.disable_level("u1", 2, 10).expect("disable");
    let err = logic.check_disable_level("u1", 2).expect_err("blocked");
    assert_eq!(err.code(), SaErrorCode::CODE_11061);
    assert!(matches!(err, SaTokenException::Framework { .. }));
}
