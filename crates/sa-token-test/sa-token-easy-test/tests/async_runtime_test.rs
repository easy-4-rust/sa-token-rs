//! Contract tests for isolated asynchronous runtimes.

use std::sync::Arc;

use sa_token_context_mock::{SaRequestForMock, SaResponseForMock, SaStorageForMock};
use sa_token_core::config::sa_token_config::SaTokenConfig;
use sa_token_core::context::model::sa_request::SaRequest;
use sa_token_core::context::model::sa_response::SaResponse;
use sa_token_core::context::model::sa_storage::SaStorage;
use sa_token_core::context::sa_token_context::SaTokenContext;
use sa_token_core::runtime::AsyncSaTokenRuntime;
use sa_token_core::stp::parameter::sa_login_parameter::SaLoginParameter;
use sa_token_core::stp::{AsyncStpUtil, StpInterface};
use sa_token_dao_memory::SaTokenDaoMemory;

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
        self.storage.delete("satoken");
    }

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
}

struct PermissionProvider;

impl StpInterface for PermissionProvider {
    fn get_permission_list(&self, login_id: &str, _login_type: &str) -> Vec<String> {
        if login_id == "alice" {
            vec!["document:read".to_string()]
        } else {
            Vec::new()
        }
    }

    fn get_role_list(&self, login_id: &str, _login_type: &str) -> Vec<String> {
        if login_id == "alice" {
            vec!["admin".to_string()]
        } else {
            Vec::new()
        }
    }
}

fn runtime() -> Arc<AsyncSaTokenRuntime> {
    runtime_with_config(SaTokenConfig::default())
}

fn runtime_with_config(config: SaTokenConfig) -> Arc<AsyncSaTokenRuntime> {
    Arc::new(
        AsyncSaTokenRuntime::new(
            Arc::new(config),
            Arc::new(SaTokenDaoMemory::new()),
            Arc::new(IsolatedContext::new()),
        )
        .with_stp_interface(Arc::new(PermissionProvider)),
    )
}

#[tokio::test]
async fn async_active_timeout_uses_java_millisecond_timestamp_semantics() {
    let config = SaTokenConfig {
        active_timeout: 1,
        ..SaTokenConfig::default()
    };
    let util = AsyncStpUtil::new("login", runtime_with_config(config));
    util.login("alice").await.expect("active-timeout login");
    assert!(util.get_token_active_timeout().await.expect("active ttl") >= 0);
    tokio::time::sleep(std::time::Duration::from_millis(2_100)).await;
    assert_eq!(
        util.get_token_active_timeout()
            .await
            .expect("expired active ttl"),
        -2
    );
    assert!(util.check_active_timeout().await.is_err());
    util.update_last_active_to_now()
        .await
        .expect("refresh active timestamp");
    util.check_active_timeout()
        .await
        .expect("refreshed token remains active");
}

#[tokio::test]
async fn runtimes_do_not_share_login_or_storage_state() {
    let alice = AsyncStpUtil::new("login", runtime());
    let bob = AsyncStpUtil::new("login", runtime());

    let alice_token = alice.login("alice").await.expect("alice login");
    let bob_token = bob.login("bob").await.expect("bob login");

    assert_eq!(alice.get_login_id().await.expect("alice id"), "alice");
    assert_eq!(bob.get_login_id().await.expect("bob id"), "bob");
    assert_eq!(
        alice
            .get_login_id_by_token(&bob_token)
            .await
            .expect("cross-runtime lookup"),
        None
    );
    assert!(
        alice
            .has_permission("document:read")
            .await
            .expect("permission query")
    );

    alice.logout().await.expect("alice logout");
    assert!(!alice.is_login().await.expect("alice state"));
    assert!(bob.is_login().await.expect("bob state"));
    assert_eq!(
        bob.get_login_id_by_token(&bob_token)
            .await
            .expect("bob token lookup"),
        Some("bob".to_string())
    );
    assert_eq!(
        alice
            .get_login_id_by_token(&alice_token)
            .await
            .expect("alice token lookup"),
        None
    );
}

#[tokio::test]
async fn async_and_sync_memory_ports_share_error_semantics() {
    let util = AsyncStpUtil::new("login", runtime());
    assert_eq!(
        util.get_login_id_by_token("missing")
            .await
            .expect("missing is not an infrastructure error"),
        None
    );
}

#[tokio::test]
async fn async_facade_covers_device_ttl_authorization_and_state_contracts() {
    let util = AsyncStpUtil::new("login", runtime());
    let parameter = SaLoginParameter::default()
        .set_device_type("desktop")
        .set_device_id("device-01")
        .set_timeout(120);
    let token = util
        .login_with_param("alice", &parameter)
        .await
        .expect("login with terminal metadata");

    assert_eq!(
        util.get_login_device_type().await.expect("device type"),
        "desktop"
    );
    assert_eq!(
        util.get_login_device_id().await.expect("device id"),
        "device-01"
    );
    assert_eq!(
        util.get_terminal_list_by_login_id("alice")
            .await
            .expect("terminal list")
            .len(),
        1
    );
    assert!(util.get_token_timeout().await.expect("token ttl") > 0);
    let token_session = util.get_token_session().await.expect("lazy token session");
    assert_eq!(token_session.session_type(), "token");
    assert_eq!(token_session.token(), token);
    util.renew_timeout(240).await.expect("renew token ttl");
    assert!(util.get_token_timeout().await.expect("renewed ttl") > 120);
    util.check_permission("document:read")
        .await
        .expect("permission contract");
    util.check_role("admin").await.expect("role contract");

    util.open_safe(60).await.expect("open safe marker");
    assert!(util.is_safe().await.expect("safe marker"));
    util.close_safe().await.expect("close safe marker");
    assert!(!util.is_safe().await.expect("closed safe marker"));

    util.switch_to("auditor").await.expect("switch identity");
    assert!(util.is_switch());
    assert_eq!(util.get_switch_login_id().as_deref(), Some("auditor"));
    util.end_switch();
    assert!(!util.is_switch());

    util.disable("alice", 60).await.expect("disable account");
    assert!(util.is_disable("alice").await.expect("disable marker"));
    assert!(util.check_disable("alice").await.is_err());
    util.untie_disable("alice")
        .await
        .expect("remove disable marker");
    assert!(!util.is_disable("alice").await.expect("untied marker"));

    util.replaced_by_login_id("alice")
        .await
        .expect("replace account token");
    assert_eq!(
        util.get_login_id_by_token(&token)
            .await
            .expect("replaced token lookup"),
        None
    );
    assert_eq!(
        util.get_terminal_list_by_login_id("alice")
            .await
            .expect("replaced terminal removed")
            .len(),
        0
    );
}

#[tokio::test]
async fn async_login_honors_share_concurrency_reserved_token_and_terminal_limit() {
    let shared = AsyncStpUtil::new("login", runtime());
    let shared_parameter = SaLoginParameter::default()
        .set_device_type("desktop")
        .set_is_share(true);
    let first = shared
        .login_with_param("alice", &shared_parameter)
        .await
        .expect("first shared login");
    let second = shared
        .login_with_param("alice", &shared_parameter)
        .await
        .expect("second shared login");
    assert_eq!(first, second);
    assert_eq!(
        shared
            .get_terminal_list_by_login_id("alice")
            .await
            .expect("deduplicated shared terminal")
            .len(),
        1
    );

    let limited = AsyncStpUtil::new("login", runtime());
    let mut tokens = Vec::new();
    for device in ["one", "two", "three"] {
        let parameter = SaLoginParameter::default()
            .set_device_type(device)
            .set_is_share(false)
            .set_max_login_count(2);
        tokens.push(
            limited
                .login_with_param("alice", &parameter)
                .await
                .expect("limited login"),
        );
    }
    assert_eq!(
        limited
            .get_login_id_by_token(&tokens[0])
            .await
            .expect("overflow token lookup"),
        None
    );
    assert_eq!(
        limited
            .get_terminal_list_by_login_id("alice")
            .await
            .expect("limited terminals")
            .len(),
        2
    );

    let exclusive = AsyncStpUtil::new("login", runtime());
    let old_token = exclusive.login("alice").await.expect("old login");
    let reserved = SaLoginParameter::default()
        .set_is_concurrent(false)
        .set_token("reserved-token");
    let new_token = exclusive
        .login_with_param("alice", &reserved)
        .await
        .expect("exclusive reserved login");
    assert_eq!(new_token, "reserved-token");
    assert_eq!(
        exclusive
            .get_login_id_by_token(&old_token)
            .await
            .expect("old token lookup"),
        None
    );
    assert_eq!(
        exclusive
            .get_terminal_list_by_login_id("alice")
            .await
            .expect("exclusive terminal")
            .len(),
        1
    );
}
