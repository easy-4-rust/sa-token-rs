//! 对应 Java：`sa-token-springboot-test`
//!
//! 框架映射：Spring Boot → axum（`sa-token-web-axum`）。
//! 本文件对齐 Java `BasicsTest` / `LoginControllerTest` 的「登录态 + 中间件可构造」冒烟路径。

use std::sync::{Arc, Mutex, MutexGuard};

use sa_token::prelude::*;
use sa_token_web_axum::SaTokenLayer;

static TEST_LOCK: Mutex<()> = Mutex::new(());

/// 初始化隔离测试环境（对应 Java `@SpringBootTest` 前的上下文准备）。
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

/// 对应 Java `BasicsTest`：登录后可读取 loginId。
#[tokio::test]
async fn basics_login_then_get_login_id() {
    let _guard = setup();
    StpUtil::login("10001").expect("login");
    assert_eq!(StpUtil::get_login_id().expect("login id"), "10001");
    StpUtil::logout().expect("logout");
}

/// 对应 Java Spring MVC Filter 注册：Axum `SaTokenLayer` 可构造。
#[test]
fn sa_token_layer_can_construct() {
    let _guard = setup();
    let _layer = SaTokenLayer::new();
}
